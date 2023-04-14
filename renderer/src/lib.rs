use js_sys::Uint8Array;
use std::str::FromStr;
use tiny_skia as sk;
use typst::{
    geom::{Color, RgbaColor},
    util::Buffer,
};
use typst_ts_core::Artifact;
use wasm_bindgen::{prelude::*, Clamped};
use web_sys::{console, ImageData};

pub mod web_font;
use web_font::WebFont;

pub(crate) mod render;

#[macro_use]
pub(crate) mod utils;

pub mod browser_world;
use browser_world::{BrowserFontSearcher, TypstBrowserWorld};

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen(module = "utils")]
extern "C" {
    async fn get_fonts() -> JsValue;
    async fn get_local_fonts() -> JsValue;
}

#[wasm_bindgen]
pub struct TypstRendererBuilder {
    searcher: BrowserFontSearcher,
}

#[wasm_bindgen]
impl TypstRendererBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<TypstRendererBuilder, JsValue> {
        Ok(Self {
            searcher: BrowserFontSearcher::new(),
        })
    }

    pub async fn add_raw_font(&mut self, font_buffer: Uint8Array) -> Result<(), JsValue> {
        // let v: JsValue =
        //     format!("raw font loading: Buffer({:?})", font_buffer.byte_length()).into();
        // console::info_1(&v);

        self.add_raw_font_internal(font_buffer.to_vec().into());
        Ok(())
    }

    pub async fn add_web_font(&mut self, font: WebFont) -> Result<(), JsValue> {
        let v: JsValue = format!("web font loading: {:?}", font).into();
        console::info_1(&v);

        self.searcher.add_web_font(font).await;

        Ok(())
    }

    pub async fn build(self) -> Result<TypstRenderer, JsValue> {
        Ok(TypstRenderer::new(
            TypstBrowserWorld::new(self.searcher).await?,
        ))
    }
}

impl TypstRendererBuilder {
    pub fn add_raw_font_internal(&mut self, font_buffer: Buffer) {
        self.searcher.add_font_data(font_buffer);
    }
}

#[wasm_bindgen]
pub struct TypstRenderer {
    world: TypstBrowserWorld,
}

#[wasm_bindgen]
impl TypstRenderer {
    fn new(world: TypstBrowserWorld) -> TypstRenderer {
        Self { world }
    }

    pub fn render(&mut self, artifact_content: String) -> Result<ImageData, JsValue> {
        let render_result = self.render_internal(artifact_content, 2., "ffffff".to_string())?;

        Ok(ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(render_result.data()),
            render_result.width(),
            render_result.height(),
        )?)
        // match result.unwrap() {
        //     Ok(document) => {
        //     }
        //     Err(errors) => Err(format!("{:?}", *errors).into()),
        // }
    }
}

impl TypstRenderer {
    pub fn render_internal(
        &self,
        artifact_content: String,
        pixel_per_pt: f32,
        fill: String,
    ) -> Result<sk::Pixmap, String> {
        // todo:
        // https://medium.com/@wl1508/avoiding-using-serde-and-deserde-in-rust-webassembly-c1e4640970ca
        let artifact: Artifact = serde_json::from_str(artifact_content.as_str()).unwrap();

        console_log!(
            "{} pages to render. font info: {:?}",
            artifact.pages.len(),
            artifact
                .fonts
                .iter()
                .map(|f| f.family.as_str()) // serde_json::to_string(f).unwrap())
                .collect::<Vec<&str>>()
                .join(", ")
        );

        let document = artifact.to_document(&self.world);
        if document.pages.len() == 0 {
            return Err("no pages in artifact".into());
        }

        Ok(crate::render::render(
            &document.pages[0],
            pixel_per_pt,
            Color::Rgba(RgbaColor::from_str(&fill)?),
        ))
        );
        Ok(ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(render.data()),
            render.width(),
            render.height(),
        )?)
        // match result.unwrap() {
        //     Ok(document) => {
        //     }
        //     Err(errors) => Err(format!("{:?}", *errors).into()),
        // }
    }
}
