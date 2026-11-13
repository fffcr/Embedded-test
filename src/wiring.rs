//! Complete the two TODOs in this file.

use embassy_stm32::{Peripherals, can::Can, usart::BufferedUart};
use embassy_stm32::gpio::{Flex, Level, Output, Speed};
use embassy_stm32::{bind_interrupts, peripherals, usart};
use static_cell::StaticCell;
use crate::swd::Swd;

bind_interrupts!(struct UartIrqs {
    UART7 => usart::BufferedInterruptHandler<peripherals::UART7>;
});

pub struct Ports {
    pub can: Can<'static>,
    pub uart: Option<BufferedUart<'static>>,
}

pub fn configure(p: Peripherals) -> (crate::board::Usb, Ports, Option<Swd>) {
    let usb = crate::board::usb(p.USB_OTG_HS, p.PA12, p.PA11);

    // TODO 1: Pick two exposed 3.3 V GPIOs and pass them to Swd::new.
    // Use Output for SWCLK (initially low), and Flex for bidirectional SWDIO.
    let swd = Some(Swd::new(
    Output::new(p.PE9, Level::Low, Speed::Low), // SWCLK
    Flex::new(p.PE13),                          // SWDIO
            ));

    static UART_TX: StaticCell<[u8; 256]> = StaticCell::new();
    static UART_RX: StaticCell<[u8; 256]> = StaticCell::new();

    let mut uart_config = usart::Config::default();
    uart_config.baudrate = 115_200;
    uart_config.data_bits = usart::DataBits::DataBits8;
    uart_config.parity = usart::Parity::ParityNone;
    uart_config.stop_bits = usart::StopBits::STOP1;

    let uart = Some(
        BufferedUart::new(
            p.UART7,
            p.PE7,
            p.PE8,
            UART_TX.init([0; 256]),
            UART_RX.init([0; 256]),
            UartIrqs,
            uart_config,
        )
        .expect("UART7 115200 8N1"),
    );

    let can = crate::board::can1(p.FDCAN1, p.PD0, p.PD1);
    (usb, Ports { can, uart }, swd)
}
