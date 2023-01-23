#![windows_subsystem = "windows"]

use std::io::prelude::*;
use std::net::{Shutdown, TcpStream};
use std::sync::{Arc, Mutex};
use windows::{core::*, Devices::Display::*, Devices::Enumeration::*, Foundation::*};

fn toggle() {
    let cmd_on = [
        0x00, 0x00, 0x00, 0x2a, 0xd0, 0xf2, 0x81, 0xf8, 0x8b, 0xff, 0x9a, 0xf7, 0xd5, 0xef, 0x94,
        0xb6, 0xc5, 0xa0, 0xd4, 0x8b, 0xf9, 0x9c, 0xf0, 0x91, 0xe8, 0xb7, 0xc4, 0xb0, 0xd1, 0xa5,
        0xc0, 0xe2, 0xd8, 0xa3, 0x81, 0xf2, 0x86, 0xe7, 0x93, 0xf6, 0xd4, 0xee, 0xdf, 0xa2, 0xdf,
        0xa2,
    ];
    let cmd_off = [
        0x00, 0x00, 0x00, 0x2a, 0xd0, 0xf2, 0x81, 0xf8, 0x8b, 0xff, 0x9a, 0xf7, 0xd5, 0xef, 0x94,
        0xb6, 0xc5, 0xa0, 0xd4, 0x8b, 0xf9, 0x9c, 0xf0, 0x91, 0xe8, 0xb7, 0xc4, 0xb0, 0xd1, 0xa5,
        0xc0, 0xe2, 0xd8, 0xa3, 0x81, 0xf2, 0x86, 0xe7, 0x93, 0xf6, 0xd4, 0xee, 0xde, 0xa3, 0xde,
        0xa3,
    ];

    let mut stream = TcpStream::connect("192.168.50.135:9999").expect("could not open connection");
    stream.write(&cmd_off).expect("could not write bytes");
    stream.read(&mut [0; 1024]).unwrap();

    std::thread::sleep(std::time::Duration::new(1, 0));

    stream.write(&cmd_on).expect("could not write bytes");
    stream.read(&mut [0; 1024]).unwrap();
    stream
        .shutdown(Shutdown::Both)
        .expect("shutdown call failed");
}

fn main()  {
    let monitor_name = "Generic Monitor (SDMU27M90*30)";
    let monitor_id = Arc::new(Mutex::new(HSTRING::from("")));
    let enumeration_complete = Arc::new(Mutex::new(false));
    let watcher = DeviceInformation::CreateWatcherAqsFilter(&DisplayMonitor::GetDeviceSelector().unwrap()).unwrap();

    watcher.Added(&TypedEventHandler::<DeviceWatcher, DeviceInformation>::new(
        {
            let monitor_id = monitor_id.clone();
            let enumeration_complete = enumeration_complete.clone();
            move |_, info| {
                let device_info = info.as_ref().expect("info");
                let device_name = device_info.Name().unwrap();
                let device_id = device_info.Id().unwrap();

                if device_name == monitor_name {
                    let mut id = monitor_id.lock().unwrap();
                    *id = device_id;

                    let complete = enumeration_complete.lock().unwrap();
                    if *complete == true {
                        println!("Monitor was readded, resetting hub.");
                        toggle();
                    }
                }
                Ok(())
            }
        },
    )).unwrap();

    watcher.Removed(
        &TypedEventHandler::<DeviceWatcher, DeviceInformationUpdate>::new({
            let monitor_id = monitor_id.clone();
            move |_, info| {
                let device_id = info.as_ref().expect("info").Id().unwrap();
                let id = monitor_id.lock().unwrap();
                if device_id == *id {
                    println!("Monitor was removed, resetting hub.");
                    toggle();
                }
                Ok(())
            }
        }),
    ).unwrap();

    watcher.Updated(
        &TypedEventHandler::<DeviceWatcher, DeviceInformationUpdate>::new(move |_, _| {
            // We only need this so that we continue to receive events.
            Ok(())
        }),
    ).unwrap();

    watcher.EnumerationCompleted(&TypedEventHandler::new(move |_, _| {
        let mut completed = enumeration_complete.lock().unwrap();
        *completed = true;
        Ok(())
    })).unwrap();

    watcher.Start().unwrap();
    std::thread::sleep(std::time::Duration::MAX);
}
