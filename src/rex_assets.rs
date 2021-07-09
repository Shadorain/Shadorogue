use rltk::rex::XpFile;

rltk::embedded_resource!(SHADOW, "../resources/shadocolor.xp");
rltk::embedded_resource!(WFC_DEMO1, "../resources/wfc-demo1.xp");
rltk::embedded_resource!(WFC_POPULATED, "../resources/wfc-populated.xp");

pub struct RexAssets {
    pub menu : XpFile,
}

impl RexAssets {
    #[allow(clippy::new_without_default)]
    pub fn new () -> RexAssets {
        rltk::link_resource!(SHADOW, "../resources/shadocolor.xp");
        rltk::link_resource!(WFC_DEMO1, "../resources/wfc-demo1.xp");
        rltk::link_resource!(WFC_POPULATED, "../resources/wfc-populated.xp");
        RexAssets { menu : XpFile::from_resource("../resources/shadocolor.xp").unwrap() }
    }
}
