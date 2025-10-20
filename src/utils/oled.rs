//! Minimal SSD1306 OLED Display Driver for STM32 BlackPill
//! 
//! Pin connections:
//! - VCC → 3V3
//! - GND → GND
//! - D0 (SCK) → PA5 (SPI1_SCK)
//! - D1 (MOSI) → PA7 (SPI1_MOSI)
//! - CS → PA4 (GPIO) - NOT USED, tied to GND
//! - DC → PB0 (GPIO)
//! - RES → PB1 (GPIO)

use stm32f4xx_hal::{
    gpio::{Pin, Output, PushPull},
    spi::Spi,
    pac::SPI1,
};

// Type aliases for our specific pin configuration
pub type DcPin = Pin<'B', 0, Output<PushPull>>;
pub type RstPin = Pin<'B', 1, Output<PushPull>>;

/// Minimal SSD1306 OLED display driver
pub struct OledDisplay {
    spi: Spi<SPI1>,
    dc: DcPin,
}

impl OledDisplay {
    /// Initialize the OLED display
    pub fn new(
        spi: Spi<SPI1>,
        dc: DcPin,
        mut rst: RstPin,
    ) -> Result<Self, &'static str> {
        let mut display = Self { spi, dc };
        
        // Hardware reset
        rst.set_low();
        cortex_m::asm::delay(100_000); // ~5ms
        rst.set_high();
        cortex_m::asm::delay(100_000); // ~5ms
        
        // Initialize SSD1306
        display.init()?;
        
        Ok(display)
    }
    
    /// Send a command byte
    fn write_cmd(&mut self, cmd: u8) -> Result<(), &'static str> {
        self.dc.set_low(); // Command mode
        self.spi.write(&[cmd]).map_err(|_| "SPI write failed")
    }
    
    /// Send data bytes
    fn write_data(&mut self, data: &[u8]) -> Result<(), &'static str> {
        self.dc.set_high(); // Data mode
        self.spi.write(data).map_err(|_| "SPI write failed")
    }
    
    /// Initialize the display with SSD1306 commands
    fn init(&mut self) -> Result<(), &'static str> {
        // Display off
        self.write_cmd(0xAE)?;
        
        // Set display clock divide ratio/oscillator frequency
        self.write_cmd(0xD5)?;
        self.write_cmd(0x80)?;
        
        // Set multiplex ratio
        self.write_cmd(0xA8)?;
        self.write_cmd(0x3F)?; // 1/64 duty
        
        // Set display offset
        self.write_cmd(0xD3)?;
        self.write_cmd(0x00)?;
        
        // Set start line
        self.write_cmd(0x40)?;
        
        // Charge pump setting
        self.write_cmd(0x8D)?;
        self.write_cmd(0x14)?; // Enable charge pump
        
        // Set memory addressing mode
        self.write_cmd(0x20)?;
        self.write_cmd(0x00)?; // Horizontal addressing mode
        
        // Set segment re-map
        self.write_cmd(0xA1)?; // Column 127 mapped to SEG0
        
        // Set COM output scan direction
        self.write_cmd(0xC8)?; // Remapped mode
        
        // Set COM pins hardware configuration
        self.write_cmd(0xDA)?;
        self.write_cmd(0x12)?;
        
        // Set contrast control
        self.write_cmd(0x81)?;
        self.write_cmd(0x7F)?;
        
        // Set pre-charge period
        self.write_cmd(0xD9)?;
        self.write_cmd(0xF1)?;
        
        // Set VCOMH deselect level
        self.write_cmd(0xDB)?;
        self.write_cmd(0x40)?;
        
        // Entire display ON (resume to RAM content display)
        self.write_cmd(0xA4)?;
        
        // Set normal display (not inverted)
        self.write_cmd(0xA6)?;
        
        // Deactivate scroll
        self.write_cmd(0x2E)?;
        
        // Display on
        self.write_cmd(0xAF)?;
        
        Ok(())
    }
    
    /// Clear the entire display
    pub fn clear(&mut self) -> Result<(), &'static str> {
        // Set column address range (0-127)
        self.write_cmd(0x21)?;
        self.write_cmd(0)?;
        self.write_cmd(127)?;
        
        // Set page address range (0-7)
        self.write_cmd(0x22)?;
        self.write_cmd(0)?;
        self.write_cmd(7)?;
        
        // Write zeros to all pixels (128 columns * 8 pages = 1024 bytes)
        let zeros = [0u8; 128];
        for _ in 0..8 {
            self.write_data(&zeros)?;
        }
        
        Ok(())
    }
    
    /// Draw a simple 5x7 character at the specified position
    pub fn draw_char(&mut self, c: char, x: u8, y: u8) -> Result<(), &'static str> {
        if x > 122 || y > 7 {
            return Ok(()); // Out of bounds
        }
        
        // Simple 5x7 font for basic ASCII characters
        let font_data = match c {
            'H' => [0x7F, 0x08, 0x08, 0x08, 0x7F],
            'e' => [0x38, 0x54, 0x54, 0x54, 0x18],
            'l' => [0x00, 0x41, 0x7F, 0x40, 0x00],
            'o' => [0x38, 0x44, 0x44, 0x44, 0x38],
            'S' => [0x32, 0x49, 0x49, 0x49, 0x26],
            'T' => [0x01, 0x01, 0x7F, 0x01, 0x01],
            'M' => [0x7F, 0x02, 0x0C, 0x02, 0x7F],
            '3' => [0x42, 0x41, 0x51, 0x69, 0x46],
            '2' => [0x42, 0x61, 0x51, 0x49, 0x46],
            'O' => [0x3E, 0x41, 0x41, 0x41, 0x3E],
            'L' => [0x7F, 0x40, 0x40, 0x40, 0x40],
            'E' => [0x7F, 0x49, 0x49, 0x49, 0x41],
            'D' => [0x7F, 0x41, 0x41, 0x22, 0x1C],
            ' ' => [0x00, 0x00, 0x00, 0x00, 0x00],
            '!' => [0x00, 0x00, 0x5F, 0x00, 0x00],
            _ => [0x7F, 0x41, 0x41, 0x41, 0x7F], // Box for unknown chars
        };
        
        // Set column address
        self.write_cmd(0x21)?;
        self.write_cmd(x)?;
        self.write_cmd(x + 5)?;
        
        // Set page address
        self.write_cmd(0x22)?;
        self.write_cmd(y)?;
        self.write_cmd(y)?;
        
        // Write character data
        self.write_data(&font_data)?;
        self.write_data(&[0x00])?; // Space between characters
        
        Ok(())
    }
    
    /// Draw a string at the specified position
    pub fn draw_text(&mut self, text: &str, x: u8, y: u8) -> Result<(), &'static str> {
        let mut pos_x = x;
        for c in text.chars() {
            if pos_x > 122 {
                break; // No more room
            }
            self.draw_char(c, pos_x, y)?;
            pos_x += 6; // 5 pixels + 1 space
        }
        Ok(())
    }
    
    /// Fill a rectangular area
    pub fn fill_rect(&mut self, x: u8, y: u8, width: u8, height: u8) -> Result<(), &'static str> {
        let end_x = (x + width).min(127);
        let end_y = (y + height / 8).min(7);
        
        // Set column address
        self.write_cmd(0x21)?;
        self.write_cmd(x)?;
        self.write_cmd(end_x)?;
        
        // Set page address
        self.write_cmd(0x22)?;
        self.write_cmd(y)?;
        self.write_cmd(end_y)?;
        
        // Fill with 0xFF (all pixels on)
        let fill = [0xFF; 128];
        for _ in y..=end_y {
            self.write_data(&fill[0..(end_x - x + 1) as usize])?;
        }
        
        Ok(())
    }
}