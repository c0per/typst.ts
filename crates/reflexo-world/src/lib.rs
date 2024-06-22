pub mod source;

pub mod entry;
pub use entry::*;

pub mod world;
pub use world::*;

pub mod font;
pub mod package;
pub mod parser;

/// Run the compiler in the system environment.
#[cfg(feature = "system")]
pub mod system;
#[cfg(feature = "system")]
pub use system::{SystemCompilerFeat, TypstSystemUniverse, TypstSystemWorld};

/// Run the compiler in the browser environment.
#[cfg(feature = "browser")]
pub(crate) mod browser;
#[cfg(feature = "browser")]
pub use browser::{BrowserCompilerFeat, TypstBrowserUniverse, TypstBrowserWorld};

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use typst::{
    diag::{At, FileResult, SourceResult},
    foundations::Bytes,
    syntax::Span,
};

use reflexo_vfs::AccessModel as VfsAccessModel;
use typst_ts_core::{
    package::Registry as PackageRegistry, typst::prelude::EcoVec, FontResolver, ImmutPath,
    TypstFileId as FileId,
};

/// Latest version of the shadow api, which is in beta.
pub trait ShadowApi {
    fn _shadow_map_id(&self, _file_id: FileId) -> FileResult<PathBuf> {
        unimplemented!()
    }

    /// Get the shadow files.
    fn shadow_paths(&self) -> Vec<Arc<Path>>;

    /// Reset the shadow files.
    fn reset_shadow(&mut self) {
        for path in self.shadow_paths() {
            self.unmap_shadow(&path).unwrap();
        }
    }

    /// Add a shadow file to the driver.
    fn map_shadow(&self, path: &Path, content: Bytes) -> FileResult<()>;

    /// Add a shadow file to the driver.
    fn unmap_shadow(&self, path: &Path) -> FileResult<()>;

    /// Add a shadow file to the driver by file id.
    /// Note: to enable this function, `ShadowApi` must implement
    /// `_shadow_map_id`.
    fn map_shadow_by_id(&self, file_id: FileId, content: Bytes) -> FileResult<()> {
        let file_path = self._shadow_map_id(file_id)?;
        self.map_shadow(&file_path, content)
    }

    /// Add a shadow file to the driver by file id.
    /// Note: to enable this function, `ShadowApi` must implement
    /// `_shadow_map_id`.
    fn unmap_shadow_by_id(&self, file_id: FileId) -> FileResult<()> {
        let file_path = self._shadow_map_id(file_id)?;
        self.unmap_shadow(&file_path)
    }
}

pub trait ShadowApiExt {
    /// Wrap the driver with a given shadow file and run the inner function.
    fn with_shadow_file<T>(
        &mut self,
        file_path: &Path,
        content: Bytes,
        f: impl FnOnce(&mut Self) -> SourceResult<T>,
    ) -> SourceResult<T>;

    /// Wrap the driver with a given shadow file and run the inner function by
    /// file id.
    /// Note: to enable this function, `ShadowApi` must implement
    /// `_shadow_map_id`.
    fn with_shadow_file_by_id<T>(
        &mut self,
        file_id: FileId,
        content: Bytes,
        f: impl FnOnce(&mut Self) -> SourceResult<T>,
    ) -> SourceResult<T>;
}

impl<C: ShadowApi> ShadowApiExt for C {
    /// Wrap the driver with a given shadow file and run the inner function.
    fn with_shadow_file<T>(
        &mut self,
        file_path: &Path,
        content: Bytes,
        f: impl FnOnce(&mut Self) -> SourceResult<T>,
    ) -> SourceResult<T> {
        self.map_shadow(file_path, content).at(Span::detached())?;
        let res: Result<T, EcoVec<typst::diag::SourceDiagnostic>> = f(self);
        self.unmap_shadow(file_path).at(Span::detached())?;
        res
    }

    /// Wrap the driver with a given shadow file and run the inner function by
    /// file id.
    /// Note: to enable this function, `ShadowApi` must implement
    /// `_shadow_map_id`.
    fn with_shadow_file_by_id<T>(
        &mut self,
        file_id: FileId,
        content: Bytes,
        f: impl FnOnce(&mut Self) -> SourceResult<T>,
    ) -> SourceResult<T> {
        let file_path = self._shadow_map_id(file_id).at(Span::detached())?;
        self.with_shadow_file(&file_path, content, f)
    }
}

/// Latest version of the world dependencies api, which is in beta.
pub trait WorldDeps {
    fn iter_dependencies(&self, f: &mut dyn FnMut(ImmutPath));
}

type CodespanResult<T> = Result<T, CodespanError>;
type CodespanError = codespan_reporting::files::Error;

/// type trait interface of [`CompilerWorld`].
pub trait CompilerFeat {
    /// Specify the font resolver for typst compiler.
    type FontResolver: FontResolver + Send + Sync + Sized;
    /// Specify the access model for VFS.
    type AccessModel: VfsAccessModel + Send + Sync + Sized;
    /// Specify the package registry.
    type Registry: PackageRegistry + Send + Sync + Sized;
}
