PA8 - Peripheral A (URXD)
PA9 - Peripheral A (UTXD)

Power management - usually peripheral 1

Interrupt configuration

Baud = Master clock / ( 16 * UART_BRGR)

Enable with UART_CR bit RXEN = 1

Data in UART_RHR, RXRDY bit of UART_SR set

Overrun indicated by OVRE bit of UART_SR, cleared setting RSTSTA of UART_CR
