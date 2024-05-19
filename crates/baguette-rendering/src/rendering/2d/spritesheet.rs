use crate::*;

#[derive(Clone)]
pub struct Tiles { index: usize, indices: Box<[u32]> }

#[derive(Clone, Default)]
pub struct SheetSection
{
    /// describes the layout of the sheet for bound checking
    /// and for index multiplication, since we need to know the amount of 
    /// sections per row we keep this value around
    layout: SpriteLayout,
    indices: Option<Tiles>,
    /// the index into the uv buffer for this section
    index: u32
}

impl SheetSection
{
    pub fn empty() -> Self
    {
        Self
        {
            layout: SpriteLayout{ rows: 1, columns: 1 },
            indices: None,
            index: 0,
        }
    }

    /// sets the index of the sheet to render with the provided row and column value
    pub fn set(&mut self, row: u32, column: u32)
    {
        // we clamp the max value possible to the length of the uv buffer, whose value is
        // determined by (rows * columns -1 ) * 4
        self.index = u32::min
        (
            column * self.layout.rows + row,
            (self.layout.rows * self.layout.columns) -1
        ) * 4
    }

    /// sets the index of the spritesheet's section to the next one
    /// or the first one if it exceeds the maximum avaiable index
    pub fn next_or_first(&mut self)
    {
        self.index = match self.indices
        {
            Some(ref mut tiles) => match tiles.indices.get(tiles.index + 1)
            {
                Some(&next_index) =>
                {
                    tiles.index += 1;
                    next_index * 4
                }
                None =>
                {
                    tiles.index = 0;
                    tiles.indices[0] * 4
                }
            }
            
            None => match self.index + 4 <= (self.layout.rows * self.layout.columns - 1) * 4
            {
                true => self.index + 4,
                false => 0
            }
        }
    }

    /// set which indices will be avaiable for playing.
    /// 
    /// # examples
    /// 
    /// accepted values are:
    /// 
    /// * [std::ops::Range]
    /// ```
    ///     section.set_indices(0..2);
    /// ```
    /// * [std::ops::RangeInclusive]
    /// ```
    ///     [std::ops::Range]
    ///     section.set_indices(0..=1);
    /// ```
    /// 
    /// * iterators of type [(u32, u32)] where the first integer represents the `row`
    /// 
    ///     while the second represents the `comumn`:
    /// 
    /// ```
    ///     section.set_indices([(0,0),(1,0),(2,0),(3,0)]);
    /// 
    ///     section.set_indices(vec![(0,1),(1,1),(2,1)]);
    /// ```
    pub fn set_indices(&mut self, items: impl IntoIndices)
    {
        self.indices = items.into_indices(self.layout);
        self.index = match &self.indices
        {
            Some(tiles) => tiles.index as u32,
            None => 0
        }
    }
}

pub trait IntoIndices 
{
    /// converts an iteration to an array of indices
    /// aligned to the correct uv instance uvs
    fn into_indices(self,layout:SpriteLayout) -> Option<Tiles> ;
}

impl IntoIndices for std::ops::Range<u32>
{
    fn into_indices(self, layout:SpriteLayout) -> Option<Tiles>
    {
        let indices = self.into_iter()
            .filter(|i| (i*4) <= (layout.rows*layout.columns-1) *4)
            .collect:: <Box<[u32]>>();

        match indices.len()>0 
        {
            true => Some(Tiles { index:0, indices }),
            false => None 
        }
    }
}

impl IntoIndices for std::ops::RangeInclusive<u32>
{
    fn into_indices(self, layout:SpriteLayout) -> Option<Tiles>
    {
        let indices = self.into_iter()
            .filter(|i| (i*4) <= (layout.rows*layout.columns-1) *4)
            .collect:: <Box<[u32]>>();

        match indices.len()>0 
        {
            true => Some(Tiles { index:0, indices }),
            false => None 
        }
    }
}

impl IntoIndices for std::ops::RangeFull
{
    fn into_indices(self, _ : SpriteLayout) -> Option<Tiles>
    {
        None 
    }
}

impl IntoIndices for ()
{
    fn into_indices(self, _ :SpriteLayout) -> Option<Tiles>
    {
        None 
    }
}

impl IntoIndices for &[u32]
{
    fn into_indices(self, layout:SpriteLayout) -> Option<Tiles>
    {
        let indices = self.iter()
            .copied()
            .filter(|i| (i*4) <= (layout.rows*layout.columns-1) *4)
            .collect::<Box<[u32]>>();

        match indices.len()>0 
        {
            true => Some(Tiles { index:0, indices }),
            false => None 
        }
    }
}

impl IntoIndices for &[(u32,u32)]
{
    fn into_indices(self, layout:SpriteLayout) -> Option<Tiles>
    {
        let indices = self.iter()
            .map(|(row,column)|column*layout.rows+row)
            .filter(|i|(i*4)<=(layout.rows*layout.columns-1)*4)
            .collect::<Box<[u32]>>();

        match indices.len()>0 
        {
            true => Some(Tiles {index:0,indices}),
            false => None
        }
    }
}

#[derive(Clone)]
pub enum SheetTiles<'a>
{
    /// like most indexing operations, the count starts from zero, so `0`
    /// returns the first tile, `1` the second, and so on
    Set(&'a [u32]),
    /// like most indexing operations, the count starts from zero, so `0`
    /// returns the first tile, `1` the second, and so on
    RowColumn(&'a[(u32,u32)]),
    /// specify the tile indices using a [`std::ops::Range`] (`start..end`)
    /// 
    /// like most indexing operations, the count starts from zero, so `0`
    /// returns the first tile, `1` the second, and so on
    Range(std::ops::Range<u32>),
    /// specify the tile indices using a [`std::ops::RangeInclusive`] (`start..=end`)
    /// 
    /// like most indexing operations, the count starts from zero, so `0`
    /// returns the first tile, `1` the second, and so on
    RangeIn(std::ops::RangeInclusive<u32>)
}

impl SheetTiles<'_>
{
    pub(crate) fn into_indices(self, layout: SpriteLayout) -> Option<spritesheet::Tiles>
    {
        use spritesheet::IntoIndices;
        
        match self
        {
            SheetTiles::Set(val) => val.into_indices(layout),
            SheetTiles::RowColumn(val) => val.into_indices(layout),
            SheetTiles::Range(val) => val.into_indices(layout),
            SheetTiles::RangeIn(val) => val.into_indices(layout),
        }
    }
}