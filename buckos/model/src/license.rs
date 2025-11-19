use serde::{Deserialize, Serialize};
use url::Url;

// # The FSF-APPROVED group includes the entire GPL-COMPATIBLE group and more.
// FSF-APPROVED @GPL-COMPATIBLE Apache-1.1 BSD-4 MPL-1.0 MPL-1.1
// # The GPL-COMPATIBLE group includes all licenses compatible with the GNU GPL.
// GPL-COMPATIBLE Apache-2.0 BSD BSD-2 GPL-2 GPL-3 LGPL-2.1 LGPL-3 X11 ZLIB

#[derive(Serialize, Deserialize, Debug)]
pub enum LicenseType {
    Apache1_1,
    Bsd4,
    Mpl1_0,
    Mpl1_1,
    Apache2_0,
    Bsd,
    Bsd2,
    Gpl2,
    Gpl3,
    Lgpl2_1,
    Lgpl3,
    X11,
    Zlib,
    Other,
    Unspecified,
    NonFree,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum LicenseOwner {
    Person,
    Organization,
    Maintainers,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct License {
    name: String,
    url: Option<Url>,
    license_type: LicenseType,
    license_text: String,
}
