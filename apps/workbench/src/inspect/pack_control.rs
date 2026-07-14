use std::path::{Component, Path, PathBuf};

#[derive(Clone, Copy)]
pub(crate) enum PackKind {
    Terrain,
    Objects,
}

pub(crate) fn validate(value: &str, kind: PackKind) -> anyhow::Result<PathBuf> {
    let path = Path::new(value);
    anyhow::ensure!(!path.is_absolute(), "pack path must be repository-relative");
    anyhow::ensure!(
        path.components()
            .all(|component| matches!(component, Component::Normal(_))),
        "pack path contains an invalid component"
    );
    let components = path.components().collect::<Vec<_>>();
    anyhow::ensure!(
        components.len() >= 3
            && components[0].as_os_str() == "out"
            && components[1].as_os_str() == "cooked",
        "pack path must be under out/cooked"
    );
    let expected = match kind {
        PackKind::Terrain => "wlt",
        PackKind::Objects => "wlr",
    };
    anyhow::ensure!(
        path.extension()
            .is_some_and(|extension| extension == expected),
        "pack must use the .{expected} extension"
    );
    Ok(path.to_path_buf())
}
