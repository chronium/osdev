use crate::arch::pic::{InterruptIndex, PICS};
use conquer_once::spin::OnceCell;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::{stream::Stream, task::AtomicWaker};
use x86_64::{instructions::port::Port, structures::idt::InterruptStackFrame};

use spin::Mutex;

#[allow(unused)]
pub enum MousePacket {
    PositionRelative(i8, i8),
    Button(u8),
}

static MOUSE_QUEUE: OnceCell<ArrayQueue<MousePacket>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

pub(crate) fn add_mouse_packet(packet: MousePacket) {
    if let Ok(queue) = MOUSE_QUEUE.try_get() {
        if let Err(_) = queue.push(packet) {
        } else {
            WAKER.wake();
        }
    } else {
        println!("WARNING: mouse queue uninitialized");
    }
}

pub struct MousePacketStream {
    _private: (),
}

impl MousePacketStream {
    pub fn new() -> Self {
        MOUSE_QUEUE
            .try_init_once(|| ArrayQueue::new(16))
            .expect("MousePacketStream::new should only be called once");
        MousePacketStream { _private: () }
    }
}

impl Stream for MousePacketStream {
    type Item = MousePacket;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let queue = MOUSE_QUEUE.try_get().expect("not initialized");
        if let Ok(packet) = queue.pop() {
            return Poll::Ready(Some(packet));
        }

        WAKER.register(&cx.waker());
        match queue.pop() {
            Ok(packet) => {
                WAKER.take();
                Poll::Ready(Some(packet))
            }
            Err(crossbeam_queue::PopError) => Poll::Pending,
        }
    }
}

static mut P64: Port<u8> = Port::new(0x64);
static mut P60: Port<u8> = Port::new(0x60);

unsafe fn wait(ty: u8) {
    let timeout = 100000;
    if ty == 0 {
        while timeout > 0 {
            if (P64.read() & 1) == 1 {
                return;
            }
        }
        return;
    } else {
        while timeout > 0 {
            if (P64.read() & 2) == 0 {
                return;
            }
        }
        return;
    }
}

unsafe fn read() -> u8 {
    wait(0);
    P60.read()
}

unsafe fn write(w: u8) {
    wait(1);
    P64.write(0xD4);
    wait(1);
    P60.write(w);
}

pub fn init() {
    unsafe {
        wait(1);
        P64.write(0xA8);

        wait(1);
        P64.write(0x20);
        wait(0);
        let status = P60.read() | 2;
        wait(1);
        P64.write(0x60);
        wait(1);
        P60.write(status);

        write(0xF6);
        read();

        write(0xF4);
        read();
    }
}

static mut MOUSE_CYCLE: u8 = 0;
static mut MOUSE_MSG: [u8; 3] = [0u8; 3];

pub static MOUSE_DX: Mutex<i8> = Mutex::new(0);
pub static MOUSE_DY: Mutex<i8> = Mutex::new(0);

pub extern "x86-interrupt" fn mouse_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    unsafe {
        x86_64::instructions::interrupts::without_interrupts(|| {
            match MOUSE_CYCLE {
                0 => {
                    MOUSE_MSG[0] = read();
                    MOUSE_CYCLE = 1;
                }
                1 => {
                    MOUSE_MSG[1] = read();
                    MOUSE_CYCLE = 2;
                }
                2 => {
                    MOUSE_MSG[2] = read();

                    let py = MOUSE_MSG[2] as i8; /*{
                                                     y @ -64..=64 => y,
                                                     _ => 0,
                                                 };*/
                    let px = MOUSE_MSG[1] as i8; /*{
                                                     x @ -64..=64 => x,
                                                     _ => 0,
                                                 };*/

                    *MOUSE_DX.lock() = px;
                    *MOUSE_DY.lock() = py;

                    add_mouse_packet(MousePacket::PositionRelative(px, py));

                    MOUSE_CYCLE = 0;
                }
                _ => unreachable!(),
            }
        });

        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Mouse.into());
    }
}
