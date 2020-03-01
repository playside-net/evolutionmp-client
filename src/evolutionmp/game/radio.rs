use crate::invoke;

pub fn get_unlocked_stations() -> u32 {
    invoke!(u32, 0xF1620ECB50E01DE7)
}

pub fn get_player_station_genre() -> u32 {
    invoke!(u32, 0xA571991A7FE6CCEB)
}

pub fn get_player_station_index() -> u8 {
    invoke!(u32, 0xE8AF77C4C06ADC93) as u8
}

pub fn get_player_station_name<'a>() -> &'a str {
    invoke!(&str, 0xF6D733C32076AD03)
}

pub fn get_player_station() -> RadioStation {
    RadioStation::from_index(get_player_station_index())
}

pub fn is_player_vehicle_radio_enabled() -> bool {
    invoke!(bool, 0x5F43D83FD6738741)
}

pub fn is_faded_out() -> bool {
    invoke!(bool, 0x0626A247D2405330)
}

pub fn is_retuning() -> bool {
    invoke!(bool, 0xA151A7394A214E65)
}

pub fn prev_station() {
    invoke!((), 0xDD6BCF9E94425DF9)
}

pub fn next_station() {
    invoke!((), 0xFF266D1D0EB1195D)
}

pub fn skip_track() {
    invoke!((), 0x6DDBBDD98E2E9C25)
}

pub fn does_player_vehicle_have_radio() -> bool {
    invoke!(bool, 0x109697E2FFBAC8A1)
}

pub struct RadioStation {
    name: String
}

impl RadioStation {
    pub fn from_index(index: u8) -> RadioStation {
        let name = invoke!(&str, 0xB28ECA15046CA8B9, index as u32).to_owned();
        RadioStation { name }
    }

    pub fn from_name<N>(name: N) -> RadioStation where N: Into<String> {
        let name = name.into();
        RadioStation { name }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn clear_custom_tracks(&self) {
        invoke!((), 0x1654F24A88A8E3FE, self.name.as_str())
    }

    pub fn set_locked(&self, locked: bool) {
        invoke!((), 0x477D9DB48F889591, self.name.as_str(), locked)
    }

    pub fn set_custom_track_list(&self, track_list_name: &str) {
        invoke!((), 0x4E404A9361F75BB2, self.name.as_str(), track_list_name, true)
    }

    pub fn set_music_only(&self, music_only: bool) {
        invoke!((), 0x774BD811F656A122, self.name.as_str(), music_only)
    }

    pub fn set_track(&self, track: &str) {
        invoke!((), 0xB39786F201FEE30B, self.name.as_str(), track)
    }

    pub fn set_track_mix(&self, mix: &str, unknown: u32) {
        invoke!((), 0x2CB0075110BE1E56 , self.name.as_str(), mix, unknown)
    }

    pub fn set_disabled(&self, disabled: bool) {
        invoke!((), 0x94F2E83EAD7E6B82, self.name.as_str(), disabled)
    }

    pub fn set_freezed(&self, freezed: bool) {
        if freezed {
            invoke!((), 0x344F393B027E38C3, self.name.as_str())
        } else {
            invoke!((), 0xFC00454CF60B91DD, self.name.as_str())
        }
    }

    pub fn unlock_track_list(&self, track_list_name: &str) {
        invoke!((), 0x031ACB6ABA18C729, self.name.as_str(), track_list_name, true)
    }

    pub fn clear_custom_track_list(&self) {
        invoke!((), 0x031ACB6ABA18C729, self.name.as_str())
    }

    pub fn make_current(&self) {
        invoke!((), 0xC69EDA28699D5107, self.name.as_str())
    }
}