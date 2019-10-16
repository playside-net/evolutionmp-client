use winapi::shared::windef::HWND;
use winapi::shared::minwindef::UINT;
use winapi::um::winuser::{MB_OK, MB_OKCANCEL, MB_ABORTRETRYIGNORE, MB_YESNOCANCEL, MB_YESNO, MB_RETRYCANCEL, MB_CANCELTRYCONTINUE, IDABORT, MB_ICONHAND, MB_ICONQUESTION, MB_ICONEXCLAMATION, MB_ICONASTERISK, MB_USERICON, MB_ICONWARNING, MB_ICONERROR, MB_ICONINFORMATION, MB_ICONSTOP, IDCANCEL, IDIGNORE, IDNO, IDOK, IDRETRY, IDYES, MessageBoxW};
use crate::win::user::MessageBoxResult::Abort;
use std::os::raw::c_int;
use std::ptr::null_mut;
use std::ffi::{OsStr, OsString};
use widestring::{WideCString};

pub unsafe fn message_box<T, C>(window: Option<HWND>, text: T, caption: C, buttons: MessageBoxButtons, icon: Option<MessageBoxIcon>) -> Option<MessageBoxResult>
    where T: Into<String>, C: Into<String> {

    let text = WideCString::from_str(text.into()).unwrap();
    let caption = WideCString::from_str(caption.into()).unwrap();
    let result = MessageBoxW(window.unwrap_or(null_mut()), text.as_ptr(), caption.as_ptr(), buttons.code() + icon.map_or(0, |i|i.code()));
    MessageBoxResult::from_code(result)
}

#[derive(Clone, Debug)]
pub enum MessageBoxButtons {
    Ok,
    OkCancel,
    AbortRetryIgnore,
    YesNoCancel,
    YesNo,
    RetryCancel,
    CancelTryContinue
}

impl MessageBoxButtons {
    pub fn code(&self) -> UINT {
        match self {
            MessageBoxButtons::Ok => MB_OK,
            MessageBoxButtons::OkCancel => MB_OKCANCEL,
            MessageBoxButtons::AbortRetryIgnore => MB_ABORTRETRYIGNORE,
            MessageBoxButtons::YesNoCancel => MB_YESNOCANCEL,
            MessageBoxButtons::YesNo => MB_YESNO,
            MessageBoxButtons::RetryCancel => MB_RETRYCANCEL,
            MessageBoxButtons::CancelTryContinue => MB_CANCELTRYCONTINUE,
        }
    }
}

#[derive(Clone, Debug)]
pub enum MessageBoxIcon {
    Hand, Question, Exclamation, Asterisk, User, Warning, Error, Information, Stop
}

impl MessageBoxIcon {
    pub fn code(&self) -> UINT {
        match self {
            MessageBoxIcon::Hand => MB_ICONHAND,
            MessageBoxIcon::Question => MB_ICONQUESTION,
            MessageBoxIcon::Exclamation => MB_ICONEXCLAMATION,
            MessageBoxIcon::Asterisk => MB_ICONASTERISK,
            MessageBoxIcon::User => MB_USERICON,
            MessageBoxIcon::Warning => MB_ICONWARNING,
            MessageBoxIcon::Error => MB_ICONERROR,
            MessageBoxIcon::Information => MB_ICONINFORMATION,
            MessageBoxIcon::Stop => MB_ICONSTOP
        }
    }
}

#[derive(Clone, Debug)]
pub enum MessageBoxResult {
    Abort, Cancel, Ignore, No, Ok, Retry, Yes
}

impl MessageBoxResult {
    pub fn code(&self) -> c_int {
        match self {
            MessageBoxResult::Abort => IDABORT,
            MessageBoxResult::Cancel => IDCANCEL,
            MessageBoxResult::Ignore => IDIGNORE,
            MessageBoxResult::No => IDNO,
            MessageBoxResult::Ok => IDOK,
            MessageBoxResult::Retry => IDRETRY,
            MessageBoxResult::Yes => IDYES,
        }
    }

    pub fn from_code(code: c_int) -> Option<MessageBoxResult> {
        return if code == IDABORT {
            Some(MessageBoxResult::Abort)
        } else if code == IDCANCEL {
            Some(MessageBoxResult::Cancel)
        } else if code == IDIGNORE {
            Some(MessageBoxResult::Ignore)
        } else if code == IDNO {
            Some(MessageBoxResult::No)
        } else if code == IDOK {
            Some(MessageBoxResult::Ok)
        } else if code == IDRETRY {
            Some(MessageBoxResult::Retry)
        } else if code == IDYES {
            Some(MessageBoxResult::Yes)
        } else {
            None
        }
    }
}

