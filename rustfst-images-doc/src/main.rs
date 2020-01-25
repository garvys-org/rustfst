use rustfst::fst_impls::VectorFst;
use rustfst::fst_traits::{MutableFst, SerializableFst};
use rustfst::semirings::{Semiring, SerializableSemiring, TropicalWeight};
use rustfst::{Arc, DrawingConfig, SymbolTable};

use failure::Fallible;
use rustfst::algorithms::{project, ProjectType};
use std::fs::{File, remove_file};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::rc::Rc;

fn generate_image<P: AsRef<Path>, F: SerializableFst>(
    path_images: P,
    fst: &F,
    name: &str,
    config: &DrawingConfig,
) -> Fallible<()>
where
    F::W: SerializableSemiring,
{
    let path_images = path_images.as_ref();
    let path_dot_file = path_images.join(format!("{}.dot", name));
    fst.draw(&path_dot_file, &config)?;

    let outputs = File::create(path_images.join(format!("{}.svg", name)))?;
    Command::new("dot")
        .args(&[
            "-Tsvg",
            path_dot_file
                .as_os_str()
                .to_str()
                .unwrap(),
        ])
        .stdout(Stdio::from(outputs))
        .spawn()?
        .wait_with_output()?;

    remove_file(path_dot_file)?;

    Ok(())
}

fn generate_project_images<P: AsRef<Path>>(path_images: P) -> Fallible<()> {
    let path_images = path_images.as_ref();
    let mut fst = VectorFst::<TropicalWeight>::new();
    let mut symt = SymbolTable::new();
    symt.add_symbols(vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"]);
    let symt = Rc::new(symt);

    fst.set_input_symbols(symt.clone());
    fst.set_output_symbols(symt);

    fst.add_states(4);

    fst.add_arc(0, Arc::new(1, 2, TropicalWeight::new(1.2), 1))?;
    fst.add_arc(1, Arc::new(3, 4, TropicalWeight::new(1.2), 2))?;
    fst.add_arc(2, Arc::new(5, 6, TropicalWeight::new(1.2), 3))?;

    fst.add_arc(1, Arc::new(7, 8, TropicalWeight::new(1.2), 1))?;
    fst.add_arc(0, Arc::new(9, 10, TropicalWeight::new(1.2), 1))?;

    fst.set_start(0)?;
    fst.set_final(3, TropicalWeight::new(0.2))?;

    let mut config = DrawingConfig::default();
    config.portrait = true;
    config.vertical = false;

    generate_image(path_images, &fst, "project.in", &config)?;
    {
        let mut fst_input_project = fst.clone();
        project(&mut fst_input_project, ProjectType::ProjectInput);
        generate_image(
            path_images,
            &fst_input_project,
            "project.out.project_input",
            &config,
        )?;
    }
    {
        let mut fst_output_project = fst.clone();
        project(&mut fst_output_project, ProjectType::ProjectOutput);
        generate_image(
            path_images,
            &fst_output_project,
            "project.out.project_output",
            &config,
        )?;
    }
    Ok(())
}

fn main() -> Fallible<()> {
    let path_crate = PathBuf::from(file!())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .canonicalize()?;
    let path_images = path_crate.join("images");

    generate_project_images(&path_images)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
