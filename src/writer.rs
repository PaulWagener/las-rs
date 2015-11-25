//! Write las files.

use std::path::Path;

use Result;
use file::File;
use header::{Header, PointFormat};
use point::Point;

/// A las writer.
///
/// This wrapper conforms to the more standard structure of requiring a filename on create, not on
/// close.
///
/// I recognize that it's pretty messy to have both this and `File`, and TODO I need to clean
/// things up.
#[derive(Debug)]
pub struct Writer<P: AsRef<Path>> {
    auto_offsets: bool,
    file: File,
    header: Header,
    path: P,
}

impl<P: AsRef<Path>> Writer<P> {
    /// Creates a new writer that will write las data to the given path.
    ///
    /// This won't actually write anything until the writer is closed.
    ///
    /// # Examples
    ///
    /// ```
    /// use las::writer::Writer;
    /// let writer = Writer::from_path("temp.las");
    /// ```
    pub fn from_path(path: P) -> Writer<P> {
        Writer {
            auto_offsets: false,
            file: File::new(),
            header: Header::new(),
            path: path,
        }
    }

    /// Sets the scale factors on a writer.
    ///
    /// # Examples
    ///
    /// ```
    /// use las::writer::Writer;
    /// let writer = Writer::from_path("temp.las").scale_factors(0.01, 0.01, 0.01);
    /// ```
    pub fn scale_factors(mut self,
                         x_scale_factor: f64,
                         y_scale_factor: f64,
                         z_scale_factor: f64)
                         -> Writer<P> {
        self.header.x_scale_factor = x_scale_factor;
        self.header.y_scale_factor = y_scale_factor;
        self.header.z_scale_factor = z_scale_factor;
        self
    }

    /// Sets the offset values for a file.
    ///
    /// # Examples
    ///
    /// ```
    /// use las::writer::Writer;
    /// let writer = Writer::from_path("temp.las").offsets(1000.0, 2000.0, 100.0);
    /// ```
    pub fn offsets(mut self, x_offset: f64, y_offset: f64, z_offset: f64) -> Writer<P> {
        self.header.x_offset = x_offset;
        self.header.y_offset = y_offset;
        self.header.z_offset = z_offset;
        self
    }

    /// Enables auto-offsetting.
    ///
    /// If auto-offsetting is enabled, this file will set the header offset values to sensible
    /// values before writing anything. This is usually easier than calculating the offsets
    /// yourself.
    ///
    /// # Examples
    ///
    /// ```
    /// use las::writer::Writer;
    /// let writer = Writer::from_path("temp.las").auto_offsets(true);
    /// ```
    pub fn auto_offsets(mut self, enable: bool) -> Writer<P> {
        self.auto_offsets = enable;
        self
    }

    /// Sets the las version for this writer.
    ///
    /// # Examples
    ///
    /// ```
    /// use las::writer::Writer;
    /// let writer = Writer::from_path("temp.las").version(1, 2);
    /// ```
    pub fn version(mut self, major: u8, minor: u8) -> Writer<P> {
        self.header.version_major = major;
        self.header.version_minor = minor;
        self
    }

    /// Sets the point format for this writer.
    ///
    /// # Examples
    ///
    /// ```
    /// use las::PointFormat;
    /// use las::writer::Writer;
    /// let writer = Writer::from_path("temp.las").point_format(PointFormat(1));
    /// ```
    pub fn point_format(mut self, point_format: PointFormat) -> Writer<P> {
        self.header.point_data_format = point_format;
        self
    }

    /// Writes a point to this writer.
    ///
    /// Note that this point won't actually be written until the writer is closed.
    ///
    /// # Examples
    ///
    /// ```
    /// use las::writer::Writer;
    /// use las::point::Point;
    /// let mut writer = Writer::from_path("temp.las");
    /// writer.write_point(Point::new());
    /// ```
    pub fn write_point(&mut self, point: Point) {
        self.file.add_point(point)
    }

    /// Closes this writer and actually writes data out to disc.
    ///
    /// Since we need to calculate some stats on the points for the header, we delay writing until
    /// the very last minute. If you don't want to hold all those points in memory, we'll need to
    /// come up with some other way to do that.
    ///
    /// This function consumes the writer.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs::remove_file;
    /// use las::writer::Writer;
    /// use las::point::Point;
    /// let mut writer = Writer::from_path("temp.las");
    /// writer.write_point(Point::new());
    /// writer.close().unwrap();
    /// remove_file("temp.las").unwrap();
    /// ```
    pub fn close(&mut self) -> Result<()> {
        self.file.set_header(self.header);
        self.file.to_path(&self.path, self.auto_offsets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::remove_file;

    use {PointFormat, File};

    #[test]
    fn builder() {
        let mut writer = Writer::from_path("builder.las")
                             .scale_factors(1.0, 2.0, 3.0)
                             .offsets(4.0, 5.0, 6.0)
                             .version(1, 2)
                             .point_format(PointFormat(1));
        writer.close().unwrap();

        let file = File::from_path("builder.las").unwrap();
        let header = file.header();
        assert_eq!(1.0, header.x_scale_factor);
        assert_eq!(2.0, header.y_scale_factor);
        assert_eq!(3.0, header.z_scale_factor);
        assert_eq!(4.0, header.x_offset);
        assert_eq!(5.0, header.y_offset);
        assert_eq!(6.0, header.z_offset);
        assert_eq!(1, header.version_major);
        assert_eq!(2, header.version_minor);
        assert_eq!(PointFormat(1), header.point_data_format);

        remove_file("builder.las").unwrap();
    }
}
