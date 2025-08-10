use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use bevy_cef_core::prelude::CefResponse;
use std::path::Path;
use std::sync::LazyLock;

pub struct LocalSchemeAssetLoaderPlugin;

impl Plugin for LocalSchemeAssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CefResponse>()
            .register_type::<CefResponseHandle>()
            .init_asset::<CefResponse>()
            .init_asset_loader::<CefResponseAssetLoader>();
    }
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component, Debug)]
pub struct CefResponseHandle(pub Handle<CefResponse>);

#[derive(Default)]
pub struct CefResponseAssetLoader;

impl AssetLoader for CefResponseAssetLoader {
    type Asset = CefResponse;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> std::result::Result<Self::Asset, Self::Error> {
        let mut body = Vec::new();
        let mime_type = get_mime_type(load_context.path())
            .unwrap_or("text/html")
            .to_string();
        reader.read_to_end(&mut body).await?;
        Ok(CefResponse {
            mime_type,
            status_code: 200,
            data: body,
        })
    }

    fn extensions(&self) -> &[&str] {
        &EXTENSIONS
    }
}

const EXTENSION_MAP: &[(&[&str], &str)] = &[
    (&["htm", "html"], "text/html"),
    (&["txt"], "text/plain"),
    (&["css"], "text/css"),
    (&["csv"], "text/csv"),
    (&["js"], "text/javascript"),
    (&["jpeg", "jpg"], "image/jpeg"),
    (&["png"], "image/png"),
    (&["gif"], "image/gif"),
    (&["bmp"], "image/bmp"),
    (&["svg"], "image/svg+xml"),
    (&["json"], "application/json"),
    (&["pdf"], "application/pdf"),
    (&["zip"], "application/zip"),
    (&["lzh"], "application/x-lzh"),
    (&["tar"], "application/x-tar"),
    (&["wasm"], "application/wasm"),
    (&["mp3"], "audio/mp3"),
    (&["mp4"], "video/mp4"),
    (&["mpeg"], "video/mpeg"),
    (&["ogg"], "audio/ogg"),
    (&["opus"], "audio/opus"),
    (&["webm"], "video/webm"),
    (&["flac"], "audio/flac"),
    (&["wav"], "audio/wav"),
    (&["m4a"], "audio/mp4"),
    (&["mov"], "video/quicktime"),
    (&["wmv"], "video/x-ms-wmv"),
    (&["mpg", "mpeg"], "video/mpeg"),
    (&["mpeg"], "video/mpeg"),
    (&["aac"], "audio/aac"),
    (&["abw"], "application/x-abiword"),
    (&["arc"], "application/x-freearc"),
    (&["avi"], "video/m-msvideo"),
    (&["azw"], "application/vnd.amazon.ebook"),
    (&["bin"], "application/octet-stream"),
    (&["bz"], "application/x-bzip"),
    (&["bz2"], "application/x-bzip2"),
    (&["csh"], "application/x-csh"),
    (&["doc"], "application/msword"),
    (
        &["docx"],
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    ),
    (&["eot"], "application/vnd.ms-fontobject"),
    (&["epub"], "application/epub+zip"),
    (&["gz"], "application/gzip"),
    (&["ico"], "image/vnd.microsoft.icon"),
    (&["ics"], "text/calendar"),
    (&["jar"], "application/java-archive"),
    (&["jpeg", "jpg"], "image/jpeg"),
    (&["mid", "midi"], "audio/midi"),
    (&["mpkg"], "application/vnd.apple.installer+xml"),
    (&["odp"], "application/vnd.oasis.opendocument.presentation"),
    (&["ods"], "application/vnd.oasis.opendocument.spreadsheet"),
    (&["odt"], "application/vnd.oasis.opendocument.text"),
    (&["oga"], "audio/ogg"),
    (&["ogv"], "video/ogg"),
    (&["ogx"], "application/ogg"),
    (&["otf"], "font/otf"),
    (&["ppt"], "application/vnd.ms-powerpoint"),
    (
        &["pptx"],
        "application/vnd.openxmlformats-officedocument.presentationml.presentation",
    ),
    (&["rar"], "application/vnd.rar"),
    (&["rtf"], "application/rtf"),
    (&["sh"], "application/x-sh"),
    (&["swf"], "application/x-shockwave-flash"),
    (&["tif", "tiff"], "image/tiff"),
    (&["ttf"], "font/ttf"),
    (&["vsd"], "application/vnd.visio"),
    (&["wav"], "audio/wav"),
    (&["weba"], "audio/webm"),
    (&["webm"], "video/web"),
    (&["woff"], "font/woff"),
    (&["woff2"], "font/woff2"),
    (&["xhtml"], "application/xhtml+xml"),
    (&["xls"], "application/vnd.ms-excel"),
    (
        &["xlsx"],
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    ),
    (&["xml"], "application/xml"),
    (&["xul"], "application/vnd.mozilla.xul+xml"),
    (&["7z"], "application/x-7z-compressed"),
];

static EXTENSIONS: LazyLock<Vec<&str>> = LazyLock::new(|| {
    EXTENSION_MAP
        .iter()
        .flat_map(|(extensions, _)| *extensions)
        .copied()
        .collect::<Vec<&str>>()
});

fn get_mime_type(path: &Path) -> Option<&str> {
    let ext = path.extension()?.to_str()?;
    EXTENSION_MAP
        .iter()
        .find(|(extensions, _)| extensions.iter().any(|e| e == &ext))
        .map(|(_, mime)| *mime)
}
