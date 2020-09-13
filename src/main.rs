use input_method_service::*;
use std::collections::HashMap;
use std::convert::TryInto;
use std::os::unix::io::IntoRawFd;
use std::time::Instant;
use std::{
    fs::OpenOptions,
    io::{Seek, SeekFrom, Write},
};
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_client::sys::client::wl_display;
use wayland_client::Proxy;
use wayland_client::{Display, GlobalManager, Main};
use wayland_protocols::unstable::text_input::v3::client::zwp_text_input_v3::{
    ContentHint, ContentPurpose,
};
use zwp_input_method::input_method_unstable_v2::zwp_input_method_manager_v2::ZwpInputMethodManagerV2;

#[derive(Clone, Debug)]
struct TestConnector {}

impl KeyboardVisability for TestConnector {
    fn show_keyboard(&self) {
        println!("Show keyboard");
    }
    fn hide_keyboard(&self) {
        println!("Hide keyboard");
    }
}

impl HintPurpose for TestConnector {
    fn set_hint_purpose(&self, content_hint: ContentHint, content_purpose: ContentPurpose) {
        println!("Hint: {:?}, Purpose: {:?}", content_hint, content_purpose);
    }
}

fn main() {
    let display = Display::connect_to_name("wayland-0").unwrap();
    let mut event_queue = display.create_event_queue();
    let attached_display = (*display).clone().attach(event_queue.token());

    let global_manager = GlobalManager::new(&attached_display);

    // Make a synchronized roundtrip to the wayland server.
    //
    // When this returns it must be true that the server has already
    // sent us all available globals.
    event_queue
        .sync_roundtrip(&mut (), |_, _, _| println!("Event received! Yeah!"))
        .unwrap();
    let seat = global_manager.instantiate_exact::<WlSeat>(1).unwrap();

    let connector = TestConnector {};
    let im_manager = get_wayland_im_manager(&global_manager);
    let im_service = IMService::new(&seat, im_manager, connector);
    loop {
        event_queue
            .dispatch(&mut (), |event, _, _| println!("Event: {:?}", event))
            .unwrap();
    }
    im_service.commit();
}

fn get_wayland_im_manager(
    global_manager: &GlobalManager,
) -> wayland_client::Main<ZwpInputMethodManagerV2> {
    global_manager
        .instantiate_exact::<ZwpInputMethodManagerV2>(1)
        .expect("Error: Your compositor does not understand the virtual_keyboard protocol!")
}
