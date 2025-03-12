use std::any::Any;

use crate::co;
use crate::decl::*;
use crate::gui::{*, privs::*};
use crate::kernel::ffi_types::*;
use crate::msg::*;
use crate::prelude::*;

#[derive(Clone)]
enum RawDlg { Raw(RawMain), Dlg(DlgMain) }

/// An user main window, which can handle events. Usually, this is the first
/// window of your application, launched directly from the `main` function. Can
/// be programmatically created or load a dialog resource from a `.res` file.
#[derive(Clone)]
pub struct WindowMain(RawDlg);

unsafe impl Send for WindowMain {}

impl AsRef<BaseWnd> for WindowMain {
	fn as_ref(&self) -> &BaseWnd {
		match &self.0 {
			RawDlg::Raw(r) => r.raw_base().base(),
			RawDlg::Dlg(d) => d.dlg_base().base(),
		}
	}
}

impl GuiWindow for WindowMain {
	fn hwnd(&self) -> &HWND {
		self.as_ref().hwnd()
	}

	fn as_any(&self) -> &dyn Any {
		self
	}
}

impl GuiParent for WindowMain {}

impl WindowMain {
	/// Instantiates a new `WindowMain` object, to be created internally with
	/// [`HWND::CreateWindowEx`](crate::prelude::user_Hwnd::CreateWindowEx).
	#[must_use]
	pub fn new(opts: WindowMainOpts) -> Self {
		Self(
			RawDlg::Raw(
				RawMain::new(opts),
			),
		)
	}

	/// Instantiates a new `WindowMain` object, to be loaded from a dialog
	/// resource with
	/// [`HINSTANCE::CreateDialogParam`](crate::prelude::user_Hinstance::CreateDialogParam).
	#[must_use]
	pub fn new_dlg(
		dlg_id: u16,
		icon_id: Option<u16>,
		accel_tbl_id: Option<u16>,
	) -> Self
	{
		Self(
			RawDlg::Dlg(
				DlgMain::new(dlg_id, icon_id, accel_tbl_id),
			),
		)
	}

	/// Physically creates the window, then runs the main application loop. This
	/// method will block until the window is closed.
	///
	/// The `cmd_show` parameter defaults to
	/// [`co::SW::SHOW`](crate::co::SW::SHOW).
	///
	/// # Panics
	///
	/// Panics if the window is already created.
	pub fn run_main(&self, cmd_show: Option<co::SW>) -> AnyResult<i32> {
		if IsWindowsVistaOrGreater()? {
			SetProcessDPIAware()?;
		}

		InitCommonControls();

		if IsWindows8OrGreater()? { // https://github.com/rodrigocfd/winsafe-examples/issues/6
			let mut b_val: BOOL = 0; // FALSE
			match unsafe {
				HPROCESS::GetCurrentProcess().SetUserObjectInformation( // SetTimer() safety
					co::UOI::TIMERPROC_EXCEPTION_SUPPRESSION,
					&mut b_val,
				)
			} {
				Err(e) if e == co::ERROR::INVALID_PARAMETER => {
					// Do nothing: Wine doesn't support SetUserObjectInformation for now.
					// https://bugs.winehq.org/show_bug.cgi?id=54951
				},
				Err(e) => panic!("TIMERPROC_EXCEPTION_SUPPRESSION failed: {e:?}"), // should never happen
				_ => {},
			}
		}

		let hinst = HINSTANCE::GetModuleHandle(None)?;
		let res = match &self.0 {
			RawDlg::Raw(r) => r.run_main(&hinst, cmd_show),
			RawDlg::Dlg(d) => d.run_main(&hinst, cmd_show),
		};

		ui_font::delete(); // cleanup
		res
	}

	/// Closes the window by posting a [`WM_CLOSE`](crate::msg::wm::Close)
	/// message. This is the safest way to close any popup window, because
	/// you'll able to process the
	/// [`wm_close`](crate::gui::events::WindowEvents::wm_close) event, just
	/// like if the user clicked the window "X" button.
	pub fn close(&self) {
		unsafe { self.hwnd().PostMessage(wm::Close {}).unwrap(); }
	}
}
