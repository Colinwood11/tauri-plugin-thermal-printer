use crate::commands_esc_pos::text::encoder::Encode;
use serde::{Deserialize, Serialize};

/// Configuracion de pagina ESC/POS y estrategia de codificacion del host.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodePage {
    pub code_page: u8,
    #[serde(default)]
    pub encode: Encode,
    #[serde(default)]
    pub use_gbk: bool,
}

impl Default for CodePage {
    fn default() -> Self {
        Self {
            code_page: 0,
            encode: Encode::AccentRemover,
            use_gbk: false,
        }
    }
}

impl CodePage {
    pub fn escpos_command(&self) -> [u8; 3] {
        [0x1B, 0x74, self.code_page]
    }
}
