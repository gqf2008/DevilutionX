//! Text Data - Dialog texts and speeches
//!
//! C++ Reference: Source/textdat.cpp, Source/textdat.h
//!
//! Implementation of all dialog texts.

use std::collections::HashMap;

/// Speech ID enumeration
///
/// C++ Reference: `_speech_id` enum in textdat.h
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i16)]
pub enum SpeechId {
    // Skeleton King quest
    King1 = 0,
    King2,
    King3,
    King4,
    King5,
    King6,
    King7,
    King8,
    King9,
    King10,
    King11,
    // Banner quest
    Banner1,
    Banner2,
    Banner3,
    Banner4,
    Banner5,
    Banner6,
    Banner7,
    Banner8,
    Banner9,
    Banner10,
    Banner11,
    Banner12,
    // Archbishop Lazarus quest
    Vile1,
    Vile2,
    Vile3,
    Vile4,
    Vile5,
    Vile6,
    Vile7,
    Vile8,
    Vile9,
    Vile10,
    Vile11,
    Vile12,
    Vile13,
    Vile14,
    // Poisoned water quest
    Poison1,
    Poison2,
    Poison3,
    Poison4,
    Poison5,
    Poison6,
    Poison7,
    Poison8,
    Poison9,
    Poison10,
    // Skeleton King Leoric quest
    Bone1,
    Bone2,
    Bone3,
    Bone4,
    Bone5,
    Bone6,
    Bone7,
    Bone8,
    // Butcher quest
    Butch1,
    Butch2,
    Butch3,
    Butch4,
    Butch5,
    Butch6,
    Butch7,
    Butch8,
    Butch9,
    Butch10,
    // Halls of the Blind quest
    Blind1,
    Blind2,
    Blind3,
    Blind4,
    Blind5,
    Blind6,
    Blind7,
    Blind8,
    // Lachdanan quest
    Veil1,
    Veil2,
    Veil3,
    Veil4,
    Veil5,
    Veil6,
    Veil7,
    Veil8,
    Veil9,
    Veil10,
    Veil11,
    // Anvil of Fury quest
    Anvil1,
    Anvil2,
    Anvil3,
    Anvil4,
    Anvil5,
    Anvil6,
    Anvil7,
    Anvil8,
    Anvil9,
    Anvil10,
    // Warlord of Blood quest
    Blood1,
    Blood2,
    Blood3,
    Blood4,
    Blood5,
    Blood6,
    Blood7,
    Blood8,
    // Warlord quest
    Warlrd1,
    Warlrd2,
    Warlrd3,
    Warlrd4,
    Warlrd5,
    Warlrd6,
    Warlrd7,
    Warlrd8,
    Warlrd9,
    // Infravision quest
    Infra1,
    Infra2,
    Infra3,
    Infra4,
    Infra5,
    Infra6,
    Infra7,
    Infra8,
    Infra9,
    Infra10,
    // Black Mushroom quest
    Mush1,
    Mush2,
    Mush3,
    Mush4,
    Mush5,
    Mush6,
    Mush7,
    Mush8,
    Mush9,
    Mush10,
    Mush11,
    Mush12,
    Mush13,
    // Diablo quest
    Doom1,
    Doom2,
    Doom3,
    Doom4,
    Doom5,
    Doom6,
    Doom7,
    Doom8,
    Doom9,
    Doom10,
    // Garbud the Weak
    Garbud1,
    Garbud2,
    Garbud3,
    Garbud4,
    // Zhar the Mad
    Zhar1,
    Zhar2,
    // Storyteller
    Story1,
    Story2,
    Story3,
    Story4,
    Story5,
    Story6,
    Story7,
    Story9,
    Story10,
    Story11,
    // Ogden (Tavern owner)
    Ogden1,
    Ogden2,
    Ogden3,
    Ogden4,
    Ogden5,
    Ogden6,
    Ogden8,
    Ogden9,
    Ogden10,
    // Pepin (Healer)
    Pepin1,
    Pepin2,
    Pepin3,
    Pepin4,
    Pepin5,
    Pepin6,
    Pepin7,
    Pepin9,
    Pepin10,
    Pepin11,
    // Gillian (Barmaid)
    Gillian1,
    Gillian2,
    Gillian3,
    Gillian4,
    Gillian5,
    Gillian6,
    Gillian7,
    Gillian9,
    Gillian10,
    // Griswold (Blacksmith)
    Griswold1,
    Griswold2,
    Griswold3,
    Griswold4,
    Griswold5,
    Griswold6,
    Griswold7,
    Griswold8,
    Griswold9,
    Griswold10,
    Griswold12,
    Griswold13,
    // Farnham (Drunk)
    Farnham1,
    Farnham2,
    Farnham3,
    Farnham4,
    Farnham5,
    Farnham6,
    Farnham8,
    Farnham9,
    Farnham10,
    Farnham11,
    Farnham12,
    Farnham13,
    // Adria (Witch)
    Adria1,
    Adria2,
    Adria3,
    Adria4,
    Adria5,
    Adria6,
    Adria7,
    Adria8,
    Adria9,
    Adria10,
    Adria12,
    Adria13,
    // Wirt (Boy)
    Wirt1,
    Wirt2,
    Wirt3,
    Wirt4,
    Wirt5,
    Wirt6,
    Wirt7,
    Wirt8,
    Wirt9,
    Wirt11,
    Wirt12,
    // Book texts
    Boner,
    Bloody,
    Blinding,
    Bloodwar,
    Mboner,
    Mbloody,
    Mblinding,
    Mbloodwar,
    Rboner,
    Rbloody,
    Rblinding,
    Rbloodwar,
    // Cow
    Cow1,
    Cow2,
    // Books
    Book11,
    Book12,
    Book13,
    Book21,
    Book22,
    Book23,
    Book31,
    Book32,
    Book33,
    // Intro
    Intro,
    // More book texts
    Hboner,
    Hbloody,
    Hblinding,
    Hbloodwar,
    Bboner,
    Bbloody,
    Bblinding,
    Bbloodwar,
    // Grave quest (Hellfire)
    Grave1,
    Grave2,
    Grave3,
    Grave4,
    Grave5,
    Grave6,
    Grave7,
    Grave8,
    Grave9,
    Grave10,
    // Farmer quest (Hellfire)
    Farmer1,
    Farmer2,
    Farmer3,
    Farmer4,
    Farmer5,
    // Girl quest (Hellfire)
    Girl1,
    Girl2,
    Girl3,
    Girl4,
    // Defiler quest (Hellfire)
    Defiler1,
    Defiler2,
    Defiler3,
    Defiler4,
    Defiler5,
    // Na-Krul quest (Hellfire)
    Nakrul1,
    Nakrul2,
    Nakrul3,
    Nakrul4,
    Nakrul5,
    // Cornerstone (Hellfire)
    Cornstn,
    // Jersey quest (Hellfire)
    Jersey1,
    Jersey2,
    Jersey3,
    Jersey4,
    Jersey5,
    Jersey6,
    Jersey7,
    Jersey8,
    Jersey9,
    // Trader
    Trader,
    // More farmer texts
    Farmer6,
    Farmer7,
    Farmer8,
    Farmer9,
    Farmer10,
    // More jersey texts
    Jersey10,
    Jersey11,
    Jersey12,
    Jersey13,
    // Skeleton journal
    Skljrn,
    // More books
    Book4,
    Book5,
    Book6,
    Book7,
    Book8,
    Book9,
    Booka,
    Bookb,
    Bookc,
    Obooka,
    Obookb,
    Obookc,
    Mbooka,
    Mbookb,
    Mbookc,
    Rbooka,
    Rbookb,
    Rbookc,
    Bbooka,
    Bbookb,
    Bbookc,
    // Dead guy
    Deadguy,
    // Extended Farnham texts
    Farnham14,
    Farnham15,
    Farnham16,
    Farnham17,
    Farnham18,
    Farnham19,
    Farnham20,
    Farnham21,
    Farnham22,
    // Extended Gillian texts
    Gillian11,
    Gillian12,
    Gillian13,
    Gillian14,
    Gillian15,
    Gillian16,
    Gillian17,
    Gillian18,
    Gillian19,
    Gillian20,
    Gillian21,
    Gillian22,
    Gillian23,
    Gillian24,
    Gillian25,
    Gillian26,
    // Extended Pepin texts
    Pepin12,
    Pepin13,
    Pepin14,
    Pepin15,
    Pepin16,
    Pepin17,
    Pepin18,
    Pepin19,
    Pepin20,
    Pepin21,
    Pepin22,
    Pepin23,
    Pepin24,
    Pepin25,
    Pepin26,
    Pepin27,
    Pepin28,
    Pepin29,
    Pepin30,
    // Extended Griswold texts
    Griswold14,
    Griswold15,
    Griswold16,
    Griswold17,
    Griswold18,
    Griswold19,
    Griswold20,
    Griswold21,
    Griswold22,
    Griswold23,
    Griswold24,
    Griswold25,
    Griswold26,
    Griswold27,
    Griswold28,
    Griswold29,
    Griswold30,
    Griswold31,
    Griswold32,
    Griswold33,
    Griswold34,
    Griswold35,
    Griswold36,
    Griswold37,
    /// Number of default text IDs
    NumDefaultTextIds,
    /// No text
    #[default]
    None = -1,
}

/// Number of default text IDs
pub const NUM_DEFAULT_TEXT_IDS: usize = SpeechId::NumDefaultTextIds as usize;

/// Speech data structure
///
/// C++ Reference: `Speech` struct in textdat.h
#[derive(Debug, Clone, Default)]
pub struct Speech {
    /// Text string
    pub txtstr: String,
    /// Whether to scroll the text
    pub scrlltxt: bool,
    /// Sound effect ID to play
    pub sfxnr: i32,
}

/// Global speeches vector
pub static mut SPEECHES: Vec<Speech> = Vec::new();

/// Additional text ID strings to indices mapping
pub static mut ADDITIONAL_TEXT_ID_STRINGS_TO_INDICES: Option<HashMap<String, i16>> = None;

/// Parse a speech ID from string
///
/// C++ Reference: `ParseSpeechId()` in textdat.cpp
pub fn parse_speech_id(value: &str) -> Result<SpeechId, String> {
    if value.is_empty() {
        return Ok(SpeechId::None);
    }
    
    // Try to match against known enum values
    // In a full implementation, this would use a string-to-enum mapping
    // For now, return an error for unknown values
    Err(format!("Unknown speech ID: {}", value))
}

/// Load text data from files
///
/// C++ Reference: `LoadTextData()` in textdat.cpp
pub fn load_text_data() {
    unsafe {
        SPEECHES.clear();
        SPEECHES.resize(NUM_DEFAULT_TEXT_IDS, Speech::default());
        
        if ADDITIONAL_TEXT_ID_STRINGS_TO_INDICES.is_none() {
            ADDITIONAL_TEXT_ID_STRINGS_TO_INDICES = Some(HashMap::new());
        } else {
            ADDITIONAL_TEXT_ID_STRINGS_TO_INDICES.as_mut().unwrap().clear();
        }
    }
    
    // TODO: Load from txtdata/text/textdat.tsv
}

/// Get a speech by ID
pub fn get_speech(id: SpeechId) -> Option<&'static Speech> {
    if id == SpeechId::None {
        return None;
    }
    
    let index = id as i16;
    if index < 0 {
        return None;
    }
    
    unsafe {
        SPEECHES.get(index as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_speech_id_values() {
        assert_eq!(SpeechId::King1 as i16, 0);
        assert_eq!(SpeechId::None as i16, -1);
    }
    
    #[test]
    fn test_speech_default() {
        let speech = Speech::default();
        assert!(speech.txtstr.is_empty());
        assert!(!speech.scrlltxt);
        assert_eq!(speech.sfxnr, 0);
    }
}
