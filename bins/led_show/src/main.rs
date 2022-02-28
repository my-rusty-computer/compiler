mod screen;

use glium::{glutin, Surface};
use mrc_emulator::error::Error;
use mrc_emulator::{Bus, Port};
use std::sync::{Arc, Mutex};

struct Io {
    data: Arc<Mutex<[u8; 8]>>,
}

impl Bus<Port> for Io {
    fn read(&self, address: Port) -> mrc_emulator::error::Result<u8> {
        if address < 8 {
            let data = self.data.lock().unwrap();
            Ok(data[address as usize])
        } else {
            Err(Error::InvalidPort(address))
        }
    }

    fn write(&mut self, address: Port, value: u8) -> mrc_emulator::error::Result<()> {
        if address < 8 {
            let mut data = self.data.lock().unwrap();
            data[address as usize] = value;
            Ok(())
        } else {
            Err(Error::InvalidPort(address))
        }
    }
}

fn main() {
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let screen = screen::Screen::new(&display);
    let data = Arc::new(Mutex::new([0_u8; 8]));

    let io = Io { data: data.clone() };

    std::thread::spawn(|| {
        let mut data = mrc_emulator::components::ram::RandomAccessMemory::with_capacity(0x100000);
        data.write(0_u32, 0xB0).unwrap();
        data.write(1_u32, 0x01).unwrap();
        data.write(2_u32, 0xE6).unwrap();
        data.write(3_u32, 0x00).unwrap();
        data.write(4_u32, 0xF4).unwrap();
        let mut cpu = mrc_emulator::cpu::CPU::new(data, io);
        cpu.start();
        println!("Done");
    });

    event_loop.run(move |event, _, control_flow| {
        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                }
                _ => return,
            },
            glutin::event::Event::NewEvents(cause) => match cause {
                glutin::event::StartCause::ResumeTimeReached { .. } => (),
                glutin::event::StartCause::Init => (),
                _ => return,
            },
            _ => return,
        }

        let next_frame_time =
            std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        // TODO: Update here.

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        let data = {
            data.lock().unwrap().clone()
        };

        println!("{:?}", data);

        screen.draw(&mut target);

        target.finish().unwrap();
    });
}