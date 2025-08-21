use crate::{BgpElem, BgpkitParser, FallibleElemIterator};
use std::io::Cursor;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmBgpParser {
    iter: FallibleElemIterator<Cursor<Vec<u8>>>,
}

#[wasm_bindgen]
impl WasmBgpParser {
    #[wasm_bindgen(constructor)]
    pub fn new(data: &[u8]) -> Result<WasmBgpParser, JsError> {
        let cursor = Cursor::new(data.to_vec());
        let parser = BgpkitParser::from_reader(cursor);
        let iter = parser.into_fallible_elem_iter();

        Ok(WasmBgpParser { iter })
    }

    #[wasm_bindgen]
    pub fn next_elem(&mut self) -> Result<Option<String>, JsError> {
        match self.iter.next() {
            Some(Ok(elem)) => {
                let json = serde_json::to_string(&elem)
                    .map_err(|e| JsError::new(&format!("Failed to serialize: {}", e)))?;
                Ok(Some(json))
            }
            Some(Err(e)) => Err(JsError::new(&format!("Parse error: {}", e))),
            None => Ok(None),
        }
    }

    #[wasm_bindgen]
    pub fn collect_all(self) -> Result<String, JsError> {
        let elems: Result<Vec<BgpElem>, _> = self.iter.collect();
        match elems {
            Ok(elems) => {
                let json = serde_json::to_string(&elems)
                    .map_err(|e| JsError::new(&format!("Failed to serialize: {}", e)))?;
                Ok(json)
            }
            Err(e) => Err(JsError::new(&format!("Parse error: {}", e))),
        }
    }
}

#[wasm_bindgen]
pub fn parse_mrt_data(data: &[u8]) -> Result<String, JsError> {
    let cursor = Cursor::new(data.to_vec());
    let parser = BgpkitParser::from_reader(cursor);

    let elems: Result<Vec<BgpElem>, _> = parser.into_fallible_elem_iter().collect();
    match elems {
        Ok(elems) => {
            let json = serde_json::to_string(&elems)
                .map_err(|e| JsError::new(&format!("Failed to serialize: {}", e)))?;
            Ok(json)
        }
        Err(e) => Err(JsError::new(&format!("Parse error: {}", e))),
    }
}
