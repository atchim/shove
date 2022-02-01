use std::{io, path::{Component, Path, PathBuf}};

pub fn rel_canon<B, P>(base: B, path: P) -> io::Result<PathBuf>
  where B: AsRef<Path>, P: AsRef<Path>
{
  let path = path.as_ref();
  Ok(match path.is_absolute() {
    false => {
      let base = base.as_ref().canonicalize()?;
      let mut canon = PathBuf::from(&base);
      path.components().for_each(|comp| {
        match comp {
          Component::CurDir => (),
          Component::ParentDir => {canon.pop();}
          Component::Normal(s) => canon.push(s),
          _ => unreachable!(),
        }
      });
      canon
    }
    true => path.to_path_buf(),
  })
}
