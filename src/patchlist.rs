// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

/// Represents a patch to be downloaded.
#[derive(Debug)]
pub struct PatchEntry {
    /// The URL of the patch file. Usually an HTTP URL.
    pub url: String,
    /// The version for this patch.
    pub version: String,
    /// How many bytes each SHA1 hash block is considering.
    pub hash_block_size: i64,
    /// Length of the patch file (in bytes.)
    pub length: i64,
    /// New size of the (game?) when the patch is installed.
    pub size_on_disk: i64,
    /// The list of SHA1 hashes.
    pub hashes: Vec<String>,

    // TODO: figure out what this is
    pub unknown_a: i32,
    // TODO: ditto
    pub unknown_b: i32,
}

/// A list of patch files the client is requested to download, and install.
#[derive(Debug)]
pub struct PatchList {
    /// The id of the patch list.
    // FIXME: this is most likely auto-generated, not set?
    pub id: String,
    /// Size of the **ffxiv** repository patches only! In bytes.
    pub patch_length: u64,
    /// The content location, usually from an HTTP URL.
    pub content_location: String,
    /// The version that was requested from the server.
    pub requested_version: String,
    /// The list of patches.
    pub patches: Vec<PatchEntry>,
}

/// The kind of patch list.
/// This must be the kind of patch list you're parsing, or else the fields will filled in the wrong order.
#[derive(PartialEq)]
#[repr(C)]
pub enum PatchListType {
    /// A boot patch list.
    Boot,
    /// A game patch ist.
    Game,
}

impl PatchList {
    pub fn from_string(patch_type: PatchListType, encoded: &str) -> Self {
        let mut patches = vec![];

        let mut patch_length = 0;
        if let Some(patch_length_index) = encoded.find("X-Patch-Length: ") {
            let rest_of_string = &encoded[patch_length_index + 16..];
            if let Some(end_of_number_index) = rest_of_string.find("\r\n") {
                let patch_length_parse: Result<u64, _> =
                    rest_of_string[0..end_of_number_index].parse();
                if let Ok(p) = patch_length_parse {
                    patch_length = p;
                }
            }
        };

        let mut content_location = String::default();
        if let Some(patch_length_index) = encoded.find("Content-Location: ") {
            let rest_of_string = &encoded[patch_length_index + 18..];
            if let Some(end_of_number_index) = rest_of_string.find("\r\n") {
                content_location = rest_of_string[0..end_of_number_index].to_string();
            }
        };

        let parts: Vec<_> = encoded.split("\r\n").collect();
        for i in 5..parts.len() - 2 {
            let patch_parts: Vec<_> = parts[i].split('\t').collect();

            if patch_type == PatchListType::Boot {
                patches.push(PatchEntry {
                    url: patch_parts[5].parse().unwrap(),
                    version: patch_parts[4].parse().unwrap(),
                    hash_block_size: 0,
                    length: patch_parts[0].parse().unwrap(),
                    size_on_disk: patch_parts[1].parse().unwrap(),
                    hashes: vec![],
                    unknown_a: 0,
                    unknown_b: 0,
                });
            } else {
                patches.push(PatchEntry {
                    url: patch_parts[8].parse().unwrap(),
                    version: patch_parts[4].parse().unwrap(),
                    hash_block_size: patch_parts[6].parse().unwrap(),
                    length: patch_parts[0].parse().unwrap(),
                    size_on_disk: patch_parts[1].parse().unwrap(),
                    hashes: patch_parts[7].split(',').map(|x| x.to_string()).collect(),
                    unknown_a: patch_parts[2].parse().unwrap(),
                    unknown_b: patch_parts[3].parse().unwrap(),
                });
            }
        }

        Self {
            id: "".to_string(),
            content_location,
            requested_version: "".to_string(),
            patch_length,
            patches,
        }
    }

    pub fn to_string(&self, patch_type: PatchListType) -> String {
        let mut str = String::new();

        // header
        str.push_str("--");
        str.push_str(&self.id);
        str.push_str("\r\n");
        str.push_str("Content-Type: application/octet-stream\r\n");
        str.push_str(&format!("Content-Location: {}\r\n", self.content_location));

        let mut total_patch_size = 0;
        for patch in &self.patches {
            total_patch_size += patch.length;
        }

        str.push_str(&format!("X-Patch-Length: {}\r\n", total_patch_size));
        str.push_str("\r\n");

        for patch in &self.patches {
            // length
            str.push_str(&patch.length.to_string());
            str.push('\t');

            // TODO: unknown value, but i *suspect* is the size of the game on disk once this patch is applied.
            // which would make sense for the launcher to check for
            str.push_str(&patch.size_on_disk.to_string());
            str.push('\t');

            // TODO: totally unknown
            str.push_str(&patch.unknown_a.to_string());
            str.push('\t');

            // TODO: unknown too
            str.push_str(&patch.unknown_b.to_string());
            str.push('\t');

            // version (e.g. 2023.09.15.0000.0000)
            str.push_str(&patch.version);
            str.push('\t');

            if patch_type == PatchListType::Game {
                // hash type
                // TODO: does this need to be configurable?
                str.push_str("sha1");
                str.push('\t');

                // hash block size
                str.push_str(&patch.hash_block_size.to_string());
                str.push('\t');

                // hashes
                str.push_str(&patch.hashes[0]);
                for hash in &patch.hashes[1..] {
                    str.push(',');
                    str.push_str(hash);
                }
                str.push('\t');
            }

            // url
            str.push_str(&patch.url);
            str.push_str("\r\n");
        }

        str.push_str("--");
        str.push_str(&self.id);
        str.push_str("--\r\n");

        str
    }

    /// Size of all the patch files by size (in bytes.)
    pub fn total_size_downloaded(&self) -> i64 {
        let mut size = 0;
        for patch in &self.patches {
            size += patch.length;
        }

        return size;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boot_patch_parsing() {
        let test_case = "--477D80B1_38BC_41d4_8B48_5273ADB89CAC\r\nContent-Type: application/octet-stream\r\nContent-Location: \
        ffxivpatch/2b5cbc63/metainfo/D2023.04.28.0000.0001.http\r\nX-Patch-Length: \
        22221335\r\n\r\n22221335\t69674819\t19\t18\t2023.09.14.0000.0001\thttp://patch-dl.ffxiv.com/boot/2b5cbc63/\
        D2023.09.14.0000.0001.patch\r\n--477D80B1_38BC_41d4_8B48_5273ADB89CAC--\r\n";

        let patch_list = PatchList::from_string(PatchListType::Boot, test_case);
        assert_eq!(patch_list.patches.len(), 1);
        assert_eq!(patch_list.patches[0].version, "2023.09.14.0000.0001");
        assert_eq!(
            patch_list.patches[0].url,
            "http://patch-dl.ffxiv.com/boot/2b5cbc63/D2023.09.14.0000.0001.patch"
        );
        assert_eq!(patch_list.patches[0].size_on_disk, 69674819);
        assert_eq!(patch_list.patch_length, 22221335);
    }

    #[test]
    fn test_game_patch_parsing() {
        let test_case = "--477D80B1_38BC_41d4_8B48_5273ADB89CAC\r\nContent-Type: application/octet-stream\r\nContent-Location: \
        ffxivpatch/4e9a232b/metainfo/2023.07.26.0000.0000.http\r\nX-Patch-Length: \
        1664916486\r\n\r\n1479062470\t44145529682\t71\t11\t2023.09.15.0000.0000\tsha1\t50000000\t1c66becde2a8cf26a99d0fc7c06f15f8bab2d87c,\
        950725418366c965d824228bf20f0496f81e0b9a,cabef48f7bf00fbf18b72843bdae2f61582ad264,53608de567b52f5fdb43fdb8b623156317e26704,\
        f0bc06cabf9ff6490f36114b25f62619d594dbe8,3c5e4b962cd8445bd9ee29011ecdb331d108abd8,88e1a2a322f09de3dc28173d4130a2829950d4e0,\
        1040667917dc99b9215dfccff0e458c2e8a724a8,149c7e20e9e3e376377a130e0526b35fd7f43df2,1bb4e33807355cdf46af93ce828b6e145a9a8795,\
        a79daff43db488f087da8e22bb4c21fd3a390f3c,6b04fadb656d467fb8318eba1c7f5ee8f030d967,a6641e1c894db961a49b70fda2b0d6d87be487a7,\
        edf419de49f42ef19bd6814f8184b35a25e9e977,c1525c4df6001b66b575e2891db0284dc3a16566,01b7628095b07fa3c9c1aed2d66d32d118020321,\
        991b137ea0ebb11bd668f82149bc2392a4cbcf52,ad3f74d4fca143a6cf507fc859544a4bcd501d85,936a0f1711e273519cae6b2da0d8b435fe6aa020,\
        023f19d8d8b3ecaaf865e3170e8243dd437a384c,2d9e934de152956961a849e81912ca8d848265ca,8e32f9aa76c95c60a9dbe0967aee5792b812d5ec,\
        dee052b9aa1cc8863efd61afc63ac3c2d56f9acc,fa81225aea53fa13a9bae1e8e02dea07de6d7052,59b24693b1b62ea1660bc6f96a61f7d41b3f7878,\
        349b691db1853f6c0120a8e66093c763ba6e3671,4561eb6f954d80cdb1ece3cc4d58cbd864bf2b50,de94175c4db39a11d5334aefc7a99434eea8e4f9,\
        55dd7215f24441d6e47d1f9b32cebdb041f2157f,2ca09db645cfeefa41a04251dfcb13587418347a\thttp://patch-dl.ffxiv.com/game/4e9a232b/\
        D2023.09.15.0000.0000.patch\r\n61259063\t44145955874\t71\t11\t2023.09.21.0000.0001\tsha1\t50000000\t88c9bbfe2af4eea7b56384baeeafd59afb47ddeb,\
        095c26e87b4d25505845515c389dd22dd429ea7e\thttp://patch-dl.ffxiv.com/game/4e9a232b/\
        D2023.09.21.0000.0001.patch\r\n63776300\t44146911186\t71\t11\t2023.09.23.0000.0001\tsha1\t50000000\tc8fc6910be12d10b39e4e6ae980d4c219cfe56a1,\
        5c0199b7147a47f620a2b50654a87c9b0cbcf43b\thttp://patch-dl.ffxiv.com/game/4e9a232b/
        D2023.09.23.0000.0001.patch\r\n32384649\t44146977234\t71\t11\t2023.09.26.0000.\
        0000\tsha1\t50000000\t519a5e46edb67ba6edb9871df5eb3991276da254\thttp://patch-dl.ffxiv.com/game/4e9a232b/\
        D2023.09.26.0000.0000.patch\r\n28434004\t44147154898\t71\t11\t2023.09.28.0000.\
        0000\tsha1\t50000000\ta08e8a071b615b0babc45a09979ab6bc70affe14\thttp://patch-dl.ffxiv.com/game/4e9a232b/\
        D2023.09.28.0000.0000.patch\r\n82378953\t5854598228\t30\t4\t2023.07.26.0000.0001\tsha1\t50000000\t07d9fecb3975028fdf81166aa5a4cca48bc5a4b0,\
        c10677985f809df93a739ed7b244d90d37456353\thttp://patch-dl.ffxiv.com/game/ex1/6b936f08/\
        D2023.07.26.0000.0001.patch\r\n29384945\t5855426508\t30\t4\t2023.09.23.0000.0001\tsha1\t50000000\tcf4970957e846e6cacfdad521252de18afc7e29b\thttp:/\
        /patch-dl.ffxiv.com/game/ex1/6b936f08/\
        D2023.09.23.0000.0001.patch\r\n136864\t5855426508\t30\t4\t2023.09.26.0000.0000\tsha1\t50000000\t3ca5e0160e1cedb7a2801048408b247095f432ea\thttp://\
        patch-dl.ffxiv.com/game/ex1/6b936f08/\
        D2023.09.26.0000.0000.patch\r\n126288\t5855426508\t30\t4\t2023.09.28.0000.0000\tsha1\t50000000\t88d201defb32366004c88b236d03278f95d9b442\thttp://\
        patch-dl.ffxiv.com/game/ex1/6b936f08/\
        D2023.09.28.0000.0000.patch\r\n49352444\t7620831756\t24\t4\t2023.09.23.0000.0001\tsha1\t50000000\t2a05a452281d119241f222f4eae43266a22560fe\thttp:/\
        /patch-dl.ffxiv.com/game/ex2/f29a3eb2/\
        D2023.09.23.0000.0001.patch\r\n301600\t7620831756\t24\t4\t2023.09.26.0000.0000\tsha1\t50000000\t215de572fe51bca45f83e19d719f52220818bc39\thttp://\
        patch-dl.ffxiv.com/game/ex2/f29a3eb2/\
        D2023.09.26.0000.0000.patch\r\n385096\t7620831756\t24\t4\t2023.09.28.0000.0000\tsha1\t50000000\t427c3fd61f2ca79698ecd6dc34e3457f6d8c01cd\thttp://\
        patch-dl.ffxiv.com/game/ex2/f29a3eb2/\
        D2023.09.28.0000.0000.patch\r\n60799419\t9737248724\t26\t4\t2023.09.23.0000.0001\tsha1\t50000000\tab064df7aec0526e8306a78e51adbbba8b129c3f,\
        4e1bb58a987b3157c16f59821bc67b1464a301e5\thttp://patch-dl.ffxiv.com/game/ex3/859d0e24/\
        D2023.09.23.0000.0001.patch\r\n555712\t9737248724\t26\t4\t2023.09.26.0000.0000\tsha1\t50000000\tdb6b1f34b0b58ca0d0621ff6cebcc63e7eb692c5\thttp://\
        patch-dl.ffxiv.com/game/ex3/859d0e24/\
        D2023.09.26.0000.0000.patch\r\n579560\t9737248724\t26\t4\t2023.09.28.0000.0000\tsha1\t50000000\t67b5d62ee8202fe045c763deea38c136c5324195\thttp://\
        patch-dl.ffxiv.com/game/ex3/859d0e24/\
        D2023.09.28.0000.0000.patch\r\n867821466\t12469514712\t40\t4\t2023.09.15.0000.0001\tsha1\t50000000\t08f6164685b4363d719d09a8d0ef0910b48ec4a1,\
        05819ea5182885b59f0dfb981ecab159f46d7343,6ab6106ce4153cac5edb26ad0aabc6ba718ee500,3c552f54cc3c101d9f1786156c1cbd9880a7c02f,\
        e0d425f6032da1ceb60ff9ca14a10e5e89f23218,245402087bf6d535cb08bbc189647e8f56301722,a3a0630bb4ddd36b532be0e0a8982dbce1bb9053,\
        eca8a1394db1e84b9ec11a99bd970e6335326da5,40546e6d37cc5ea21d26c2e00a11f46a13d99a77,f41f312ad72ee65dc97645c8f7426c35970187ca,\
        e5a9966528ecab5a51059de3d5cd83a921c73a1c,c03855127d135d22c65e34e615eddbe6f38769e9,befab30a77c14743f53b20910b564bb6a97dfe86,\
        dcce8ea707f03606b583d51d947a4cf99b52635e,c4a33a8c51a047706b65887bed804ec6c2c29016,a17bc8bd8709c2a0c5725c134101132d3536e67d,\
        d2b277de55a65697d80cfe8ee46199a8d7482c30,6b97cc2862c6f8f5d279be5f28cc13ed011763e5\thttp://patch-dl.ffxiv.com/game/ex4/1bf99b87/\
        D2023.09.15.0000.0001.patch\r\n17717567\t12471821656\t40\t4\t2023.09.23.0000.0001\tsha1\t50000000\tda144d5c1c173ef1d98e4e7b558414ae53bcd392\thttp:\
        //patch-dl.ffxiv.com/game/ex4/1bf99b87/\
        D2023.09.23.0000.0001.patch\r\n3117253\t12471859944\t40\t4\t2023.09.26.0000.0000\tsha1\t50000000\t7acdab61e99d69ffa53e9136f65a1a1d3b33732b\thttp:/\
        /patch-dl.ffxiv.com/game/ex4/1bf99b87/\
        D2023.09.26.0000.0000.patch\r\n4676853\t12471859944\t40\t4\t2023.09.28.0000.0000\tsha1\t50000000\tce26ccb2115af612ccd4c42c1a27ef2ec925c81e\thttp:/\
        /patch-dl.ffxiv.com/game/ex4/1bf99b87/D2023.09.28.0000.0000.patch\r\n--477D80B1_38BC_41d4_8B48_5273ADB89CAC--\r\n";

        let patch_list = PatchList::from_string(PatchListType::Game, test_case);
        assert_eq!(patch_list.patches.len(), 19);
        assert_eq!(patch_list.patches[5].version, "2023.07.26.0000.0001");
        assert_eq!(
            patch_list.patches[5].url,
            "http://patch-dl.ffxiv.com/game/ex1/6b936f08/D2023.07.26.0000.0001.patch"
        );
        assert_eq!(patch_list.patches[5].size_on_disk, 5854598228);
        assert_eq!(patch_list.patch_length, 1664916486);
    }

    #[test]
    fn test_boot_patch_output() {
        let test_case = "--477D80B1_38BC_41d4_8B48_5273ADB89CAC\r\nContent-Type: application/octet-stream\r\nContent-Location: \
        ffxivpatch/2b5cbc63/metainfo/D2023.04.28.0000.0001.http\r\nX-Patch-Length: \
        22221335\r\n\r\n22221335\t69674819\t19\t18\t2023.09.14.0000.0001\thttp://patch-dl.ffxiv.com/boot/2b5cbc63/\
        D2023.09.14.0000.0001.patch\r\n--477D80B1_38BC_41d4_8B48_5273ADB89CAC--\r\n";

        let patch_list = PatchList {
            id: "477D80B1_38BC_41d4_8B48_5273ADB89CAC".to_string(),
            requested_version: "D2023.04.28.0000.0001".to_string(),
            content_location: "ffxivpatch/2b5cbc63/metainfo/D2023.04.28.0000.0001.http".to_string(),
            patches: vec![PatchEntry {
                url: "http://patch-dl.ffxiv.com/boot/2b5cbc63/D2023.09.14.0000.0001.patch"
                    .to_string(),
                version: "2023.09.14.0000.0001".to_string(),
                hash_block_size: 0,
                length: 22221335,
                size_on_disk: 69674819,
                hashes: vec![],
                unknown_a: 19,
                unknown_b: 18,
            }],
            patch_length: 22221335,
        };

        assert_eq!(patch_list.to_string(PatchListType::Boot), test_case);
    }

    #[test]
    fn test_game_patch_output() {
        let test_case = "--477D80B1_38BC_41d4_8B48_5273ADB89CAC\r\nContent-Type: application/octet-stream\r\nContent-Location: \
        ffxivpatch/4e9a232b/metainfo/2023.07.26.0000.0000.http\r\nX-Patch-Length: \
        1479062470\r\n\r\n1479062470\t44145529682\t71\t11\t2023.09.15.0000.0000\tsha1\t50000000\t1c66becde2a8cf26a99d0fc7c06f15f8bab2d87c,\
        950725418366c965d824228bf20f0496f81e0b9a,cabef48f7bf00fbf18b72843bdae2f61582ad264,53608de567b52f5fdb43fdb8b623156317e26704,\
        f0bc06cabf9ff6490f36114b25f62619d594dbe8,3c5e4b962cd8445bd9ee29011ecdb331d108abd8,88e1a2a322f09de3dc28173d4130a2829950d4e0,\
        1040667917dc99b9215dfccff0e458c2e8a724a8,149c7e20e9e3e376377a130e0526b35fd7f43df2,1bb4e33807355cdf46af93ce828b6e145a9a8795,\
        a79daff43db488f087da8e22bb4c21fd3a390f3c,6b04fadb656d467fb8318eba1c7f5ee8f030d967,a6641e1c894db961a49b70fda2b0d6d87be487a7,\
        edf419de49f42ef19bd6814f8184b35a25e9e977,c1525c4df6001b66b575e2891db0284dc3a16566,01b7628095b07fa3c9c1aed2d66d32d118020321,\
        991b137ea0ebb11bd668f82149bc2392a4cbcf52,ad3f74d4fca143a6cf507fc859544a4bcd501d85,936a0f1711e273519cae6b2da0d8b435fe6aa020,\
        023f19d8d8b3ecaaf865e3170e8243dd437a384c,2d9e934de152956961a849e81912ca8d848265ca,8e32f9aa76c95c60a9dbe0967aee5792b812d5ec,\
        dee052b9aa1cc8863efd61afc63ac3c2d56f9acc,fa81225aea53fa13a9bae1e8e02dea07de6d7052,59b24693b1b62ea1660bc6f96a61f7d41b3f7878,\
        349b691db1853f6c0120a8e66093c763ba6e3671,4561eb6f954d80cdb1ece3cc4d58cbd864bf2b50,de94175c4db39a11d5334aefc7a99434eea8e4f9,\
        55dd7215f24441d6e47d1f9b32cebdb041f2157f,2ca09db645cfeefa41a04251dfcb13587418347a\thttp://patch-dl.ffxiv.com/game/4e9a232b/\
        D2023.09.15.0000.0000.patch\r\n--477D80B1_38BC_41d4_8B48_5273ADB89CAC--\r\n";

        let patch_list = PatchList {
            id: "477D80B1_38BC_41d4_8B48_5273ADB89CAC".to_string(),
            requested_version: "2023.07.26.0000.0000".to_string(),
            content_location: "ffxivpatch/4e9a232b/metainfo/2023.07.26.0000.0000.http".to_string(),
            patches: vec![PatchEntry {
                url: "http://patch-dl.ffxiv.com/game/4e9a232b/D2023.09.15.0000.0000.patch"
                    .to_string(),
                version: "2023.09.15.0000.0000".to_string(),
                hash_block_size: 50000000,
                length: 1479062470,
                size_on_disk: 44145529682,
                unknown_a: 71,
                unknown_b: 11,
                hashes: vec![
                    "1c66becde2a8cf26a99d0fc7c06f15f8bab2d87c".to_string(),
                    "950725418366c965d824228bf20f0496f81e0b9a".to_string(),
                    "cabef48f7bf00fbf18b72843bdae2f61582ad264".to_string(),
                    "53608de567b52f5fdb43fdb8b623156317e26704".to_string(),
                    "f0bc06cabf9ff6490f36114b25f62619d594dbe8".to_string(),
                    "3c5e4b962cd8445bd9ee29011ecdb331d108abd8".to_string(),
                    "88e1a2a322f09de3dc28173d4130a2829950d4e0".to_string(),
                    "1040667917dc99b9215dfccff0e458c2e8a724a8".to_string(),
                    "149c7e20e9e3e376377a130e0526b35fd7f43df2".to_string(),
                    "1bb4e33807355cdf46af93ce828b6e145a9a8795".to_string(),
                    "a79daff43db488f087da8e22bb4c21fd3a390f3c".to_string(),
                    "6b04fadb656d467fb8318eba1c7f5ee8f030d967".to_string(),
                    "a6641e1c894db961a49b70fda2b0d6d87be487a7".to_string(),
                    "edf419de49f42ef19bd6814f8184b35a25e9e977".to_string(),
                    "c1525c4df6001b66b575e2891db0284dc3a16566".to_string(),
                    "01b7628095b07fa3c9c1aed2d66d32d118020321".to_string(),
                    "991b137ea0ebb11bd668f82149bc2392a4cbcf52".to_string(),
                    "ad3f74d4fca143a6cf507fc859544a4bcd501d85".to_string(),
                    "936a0f1711e273519cae6b2da0d8b435fe6aa020".to_string(),
                    "023f19d8d8b3ecaaf865e3170e8243dd437a384c".to_string(),
                    "2d9e934de152956961a849e81912ca8d848265ca".to_string(),
                    "8e32f9aa76c95c60a9dbe0967aee5792b812d5ec".to_string(),
                    "dee052b9aa1cc8863efd61afc63ac3c2d56f9acc".to_string(),
                    "fa81225aea53fa13a9bae1e8e02dea07de6d7052".to_string(),
                    "59b24693b1b62ea1660bc6f96a61f7d41b3f7878".to_string(),
                    "349b691db1853f6c0120a8e66093c763ba6e3671".to_string(),
                    "4561eb6f954d80cdb1ece3cc4d58cbd864bf2b50".to_string(),
                    "de94175c4db39a11d5334aefc7a99434eea8e4f9".to_string(),
                    "55dd7215f24441d6e47d1f9b32cebdb041f2157f".to_string(),
                    "2ca09db645cfeefa41a04251dfcb13587418347a".to_string(),
                ],
            }],
            patch_length: 1479062470,
        };

        assert_eq!(patch_list.to_string(PatchListType::Game), test_case);
    }
}
