use {
    metrohash::{MetroHashMap},
    rusttype::{Font},

    {Error, ResourceError}
};

pub struct Resources {
    fonts: MetroHashMap<FontId, Font<'static>>,
    next_id: u32,
}

impl Resources {
    pub(crate) fn new() -> Self {
        Resources {
            fonts: MetroHashMap::default(),
            next_id: 0,
        }
    }

    pub fn add_font(&mut self, bytes: Vec<u8>) -> FontId {
        let font = Font::from_bytes(bytes).unwrap();

        let id = FontId { id: self.next_id };
        self.next_id += 1;
        self.fonts.insert(id, font);

        id
    }

    pub fn font(&self, font_id: FontId) -> Result<&Font, Error> {
        self.fonts.get(&font_id)
            .ok_or(Error::Resource(ResourceError::InvalidId))
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct FontId { id: u32 }
