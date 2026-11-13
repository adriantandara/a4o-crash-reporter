use windows::Win32::Foundation::{HWND, LPARAM, S_OK, WPARAM};
use windows::Win32::UI::Controls::{
    TASKDIALOG_BUTTON, TASKDIALOG_NOTIFICATIONS, TASKDIALOGCONFIG, TASKDIALOGCONFIG_0, TDF_ALLOW_DIALOG_CANCELLATION,
    TDF_ENABLE_HYPERLINKS, TDF_EXPANDED_BY_DEFAULT, TDF_USE_HICON_MAIN, TDF_VERIFICATION_FLAG_CHECKED,
    TDN_HYPERLINK_CLICKED, TaskDialogIndirect,
};
use windows::core::{BOOL, HRESULT, HSTRING, PCWSTR};

use super::icon;
use crate::config;
use crate::storage::settings;
use crate::system::{autostart, shell, time};

const ACCEPT: i32 = 100;
const DECLINE: i32 = 101;

const INSTRUCTION: &str = "Ajută ALL4ONE să repare crash-urile";

const CONTENT: &str = "Când FiveM se închide cu eroare în timp ce joci pe ALL4ONE, aplicația poate trimite \
automat raportul de crash echipei ALL4ONE, ca să găsim cauza (asset-uri, scripturi, resurse).\n\n\
Crash-urile de pe alte servere sunt ignorate. Poți pune aplicația pe pauză sau o poți închide oricând \
din iconița de lângă ceas.";

const DETAILS: &str = "Ce face, doar după ce accepți:\n\
• verifică dacă FiveM rulează și dacă apare fereastra lui de eroare;\n\
• citește din FiveM ultimul server pe care ai jucat, ca să ignore celelalte servere;\n\
• apasă „Save information” în fereastra de eroare și o închide după salvare (poți dezactiva închiderea);\n\
• din arhiva salvată de FiveM în Descărcări trimite doar fișierul .dmp, crashometry.json (fără Rockstar ID) \
și jurnalul CitizenFX, din care elimină numele și calea contului tău de Windows.\n\n\
Ce conține un dump: o parte din memoria jocului din momentul crash-ului. Poate include numele tău din joc, \
mesaje recente sau identificatori de sesiune FiveM. Nu conține fișierele tale personale.\n\n\
Unde ajung datele: criptat (HTTPS) la guard.all4one.ro. Raportul este legat de contul tău de pe server, \
este văzut doar de echipa de dezvoltare, este folosit doar pentru depanare și se șterge automat după 90 de zile.\n\n\
Ce NU face: nu se injectează în FiveM sau GTA V, nu modifică fișierele jocului, nu citește alte fișiere \
și nu cere drepturi de administrator.\n\n\
Poți retrage acordul oricând din meniul aplicației. Datele locale se șterg imediat, iar pentru rapoartele deja \
trimise poți cere ștergerea pe Discord-ul ALL4ONE.";

pub fn ensure() -> bool {
    if settings::get_u64(settings::CONSENT_VERSION) == Some(config::CONSENT_VERSION) {
        return true;
    }
    match ask() {
        Some(start_with_windows) => {
            settings::set(settings::CONSENT_VERSION, config::CONSENT_VERSION);
            settings::set(settings::CONSENT_DATE, time::local_timestamp());
            settings::set(settings::AUTOSTART, start_with_windows);
            autostart::set_enabled(start_with_windows);
            true
        }
        None => false,
    }
}

fn ask() -> Option<bool> {
    let title = HSTRING::from(config::APP_NAME);
    let instruction = HSTRING::from(INSTRUCTION);
    let content = HSTRING::from(CONTENT);
    let details = HSTRING::from(DETAILS);
    let expand = HSTRING::from("Ce face exact aplicația");
    let collapse = HSTRING::from("Ascunde detaliile");
    let verification = HSTRING::from("Pornește odată cu Windows");
    let footer = HSTRING::from(format!(
        "Open source, licență MIT · <a href=\"{}\">Codul sursă</a> · <a href=\"{}\">Politica de confidențialitate</a>",
        config::REPOSITORY_URL,
        config::PRIVACY_URL
    ));
    let accept = HSTRING::from("Accept");
    let decline = HSTRING::from("Refuz");
    let buttons = [
        TASKDIALOG_BUTTON {
            nButtonID: ACCEPT,
            pszButtonText: PCWSTR(accept.as_ptr()),
        },
        TASKDIALOG_BUTTON {
            nButtonID: DECLINE,
            pszButtonText: PCWSTR(decline.as_ptr()),
        },
    ];

    let dialog = TASKDIALOGCONFIG {
        cbSize: size_of::<TASKDIALOGCONFIG>() as u32,
        dwFlags: TDF_ENABLE_HYPERLINKS
            | TDF_USE_HICON_MAIN
            | TDF_ALLOW_DIALOG_CANCELLATION
            | TDF_EXPANDED_BY_DEFAULT
            | TDF_VERIFICATION_FLAG_CHECKED,
        pszWindowTitle: PCWSTR(title.as_ptr()),
        Anonymous1: TASKDIALOGCONFIG_0 {
            hMainIcon: icon::large(),
        },
        pszMainInstruction: PCWSTR(instruction.as_ptr()),
        pszContent: PCWSTR(content.as_ptr()),
        cButtons: buttons.len() as u32,
        pButtons: buttons.as_ptr(),
        nDefaultButton: DECLINE,
        pszVerificationText: PCWSTR(verification.as_ptr()),
        pszExpandedInformation: PCWSTR(details.as_ptr()),
        pszExpandedControlText: PCWSTR(collapse.as_ptr()),
        pszCollapsedControlText: PCWSTR(expand.as_ptr()),
        pszFooter: PCWSTR(footer.as_ptr()),
        pfCallback: Some(on_notification),
        cxWidth: 300,
        ..Default::default()
    };

    let mut pressed = 0;
    let mut start_with_windows = BOOL(0);
    unsafe { TaskDialogIndirect(&dialog, Some(&mut pressed), None, Some(&mut start_with_windows)) }.ok()?;
    (pressed == ACCEPT).then_some(start_with_windows.as_bool())
}

unsafe extern "system" fn on_notification(
    _hwnd: HWND,
    notification: TASKDIALOG_NOTIFICATIONS,
    _wparam: WPARAM,
    lparam: LPARAM,
    _data: isize,
) -> HRESULT {
    if notification == TDN_HYPERLINK_CLICKED
        && let Ok(url) = unsafe { PCWSTR(lparam.0 as *const u16).to_string() }
    {
        shell::open_url(&url);
    }
    S_OK
}
