use std::ops::{Deref, DerefMut};
use std::slice;

use crate::*;

pub struct SpriteSheet
{
    pub inner: Sprite,
    sections: FastIndexMap<u8, Vec<SliceSection>>,
}

impl Deref for SpriteSheet
{
    type Target = Sprite;

    fn deref(&self) -> &Self::Target
    {
        &self.inner
    }
}

impl DerefMut for SpriteSheet
{ 
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.inner
    }
}

impl SpriteSheet
{
    pub fn new(renderer: &mut crate::Renderer, loader: SpriteSheetBuilder) -> Self
    {
        Self
        {
            inner: Sprite::new(renderer, loader.inner),
            sections: loader.sections,
        }
    }

    pub fn iter_layer(&mut self, layer: u8) -> Iter
    {
        let other = self.sections[&layer].iter();

        let iter = self.inner.iter_layer(layer).zip(other);

        Iter(iter)
    }

    /// iters the layer mutably.
    /// 
    /// [`Self::iter_layer`] is faster if you need to iter immutably 
    pub fn iter_layer_mut(&mut self, layer: u8) -> IterMut
    {
        IterMut
        {
            layers_iter: self.inner.iter_layer_mut(layer),
            slices_iter: self.sections[&layer].iter_mut(),
        }
    }
}

pub struct Iter<'a>(std::iter::Zip<sprite::IterLayer<'a>, slice::Iter<'a, SliceSection>>);

impl<'a> Iterator for Iter<'a>
{
    type Item = (&'a SpriteInstance, &'a SliceSection);
    
    fn next(&mut self) -> Option<Self::Item>
    {
        self.0.next()
    }
}

pub struct IterMut<'a>
{
    layers_iter: sprite::IterLayerMut<'a>,
    slices_iter: slice::IterMut<'a, SliceSection>
}

impl<'a> Iterator for IterMut<'a>
{
    type Item = SpriteSheetInstance<'a>;

    fn next(&mut self) -> Option<Self::Item>
    {
        Some
        (
            SpriteSheetInstance
            (
                self.layers_iter.next()?,
                self.slices_iter.next()?
            )
        )
    }
}

/// describes the type of sprite you want to create
pub struct SpriteSheetBuilder
{
    inner: SpriteBuilder,
    sections: FastIndexMap<u8, Vec<SliceSection>>,
}

impl SpriteSheetBuilder
{
    pub fn new
    (
        path: impl Into<std::path::PathBuf>,
        rows: u32,
        columns: u32
    )
    -> Self
    {
        let inner = SpriteBuilder::new(path)
            .tiled_atlas(rows, columns);

        Self
        {
            inner,
            sections: FastIndexMap::default(),
        }
    }
    /// Example
    /// 
    /// ```
    ///     SpriteSheetBuilder::new
    ///     (
    ///         "assets/green dude sheet.png",
    ///         6, 5
    ///     )
    ///     .set_layer::<0>
    ///     (
    ///         [(Default::default(), SheetSlices::Range(19..22))]
    ///     )
    /// ```
    pub fn set_layer<'a, const LAYER: u8>
    (
        mut self,
        instances: impl IntoIterator<Item = (SpriteInstance, SheetSlices<'a>)>
    )
    -> Self
    {
        let (instances, slices): (Vec<_>, Vec<_>) = instances.into_iter().unzip();

        self.inner = self.inner.set_layer::<LAYER>(instances);

        let SpriteBuilder { rows, columns, .. } = self.inner;

        let slices = slices.into_iter().map(|tiles|
        {
            SliceSection
            {
                rows,
                columns,
                indices: tiles.into_indices(rows, columns),
                index: 0
            }
        }).collect::<Vec<_>>();

        match slices.is_empty()
        {
            false => self.sections.insert(LAYER, slices),
            true => self.sections.insert(LAYER, vec![SliceSection
            {
                rows,
                columns,
                ..Default::default()
            }])
        };

        Self
        {
            inner: self.inner,
            sections: self.sections,
        }
    }
}

/// contains the specific indices to display
#[derive(Clone)]
pub struct Tiles { index: usize, indices: Box<[u32]> }

/// contains the section to display from the [`SpriteSheet`], along with the next sections that will be displayed
#[derive(Clone)]
pub struct SliceSection
{
    rows: u32,
    columns: u32,

    /// contains the specific indices to display, if this is [None]
    /// then all the sections of the [SpriteSheet] are avaiable for display
    indices: Option<Tiles>,
    /// the index into the uv buffer for this section,
    /// 
    /// this needs to be passed to the [SpriteInstance]'s uv_index to change it's uv value 
    index: u32
}

impl Default for SliceSection
{
    /// returns an unsliced section with no indices
    fn default() -> Self
    {
        Self
        {
            rows: 1,
            columns: 1,
    
            indices: None,
            index: 0,
        }
    }
}

impl SliceSection
{
    /// sets the index of the sheet to render with the provided row and column value
    pub fn set(&mut self, row: u32, column: u32)
    {
        // we clamp the max value possible to the length of the uv buffer, the value is
        // clamped to rows * columns -1
        self.index = u32::min
        (
            column * self.rows + row,
            (self.rows * self.columns) -1
        )
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
                    next_index
                }
                None =>
                {
                    tiles.index = 0;
                    tiles.indices[0]
                }
            }
            None => match self.index < (self.rows * self.columns)
            {
                true => self.index + 1,
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
        self.indices = items.into_indices(self.rows, self.columns);
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
    fn into_indices(self, rows: u32, columns: u32) -> Option<Tiles>;
}

impl IntoIndices for std::ops::Range<u32>
{
    fn into_indices(self, rows: u32, columns: u32) -> Option<Tiles>
    {
        let indices = self.into_iter()
            .filter(|i| *i < rows*columns)
            .collect::<Box<[u32]>>();

        match indices.len()>0 
        {
            true => Some(Tiles { index:0, indices }),
            false => None 
        }
    }
}

impl IntoIndices for std::ops::RangeInclusive<u32>
{
    fn into_indices(self, rows: u32, columns: u32) -> Option<Tiles>
    {
        let indices = self.into_iter()
            .filter(|i| *i < rows*columns)
            .collect::<Box<[u32]>>();

        match indices.len()>0 
        {
            true => Some(Tiles { index:0, indices }),
            false => None 
        }
    }
}

impl IntoIndices for std::ops::RangeFull
{
    fn into_indices(self, _: u32, _: u32) -> Option<Tiles>
    {
        None 
    }
}

impl IntoIndices for ()
{
    fn into_indices(self,  _: u32, _: u32) -> Option<Tiles>
    {
        None 
    }
}

impl IntoIndices for &[u32]
{
    fn into_indices(self, rows: u32, columns: u32) -> Option<Tiles>
    {
        let indices = self.iter()
            .copied()
            .filter(|i| *i < rows*columns-1)
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
    fn into_indices(self,  rows: u32, columns: u32) -> Option<Tiles>
    {
        let indices = self.iter()
            .map(|(row,column)| column * rows + row)
            .filter(|i| *i < rows*columns-1)
            .collect::<Box<[u32]>>();

        match indices.len() > 0 
        {
            true => Some(Tiles {index:0,indices}),
            false => None
        }
    }
}

#[derive(Clone)]
pub enum SheetSlices<'a>
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
    RangeIn(std::ops::RangeInclusive<u32>),
    All
}

impl SheetSlices<'_>
{
    pub(crate) fn into_indices(self, rows: u32, columns: u32) -> Option<spritesheet::Tiles>
    {
        use spritesheet::IntoIndices;
        
        match self
        {
            SheetSlices::Set(val) => val.into_indices(rows, columns),
            SheetSlices::RowColumn(val) => val.into_indices(rows, columns),
            SheetSlices::Range(val) => val.into_indices(rows, columns),
            SheetSlices::RangeIn(val) => val.into_indices(rows, columns),
            SheetSlices::All => None,
        }
    }
}

pub struct SpriteSheetInstance<'a>
(
    &'a mut SpriteInstance,
    &'a mut SliceSection
);

impl SpriteSheetInstance<'_>
{
    pub fn next_or_first(&mut self)
    {
        self.1.next_or_first()
    }
    
    pub fn set(&mut self, row: u32, column: u32)
    {
        self.1.set(row, column)
    }
    
    pub fn set_indices(&mut self, items: impl IntoIndices)
    {
        self.1.set_indices(items)
    }
}

impl<'a> Deref for SpriteSheetInstance<'a>
{
    type Target = &'a mut SpriteInstance;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}

impl DerefMut for SpriteSheetInstance<'_>
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.0
    }
}

impl Drop for SpriteSheetInstance<'_>
{
    fn drop(&mut self)
    {
        // here we pass the possibly modified uv section to the sprite
        self.0.uv_idx = self.1.index
    }
}