use thiserror::Error;
use itertools::Itertools;


#[derive(Error, Debug, PartialEq, Eq)]
pub enum ScoringError {
    #[error("Missing line")]
    MissingLine,
    #[error("Expected a number")]
    ExpectedANumber,
    #[error("Expected a library id")]
    MissingLibraryId,
    #[error("Wrong library id format")]
    WrongFormatLibraryId,
    #[error("Expected the number of books at library signup")]
    MissingNumOfBooksForLibrarySignup,
    #[error("Wrong format of number of books at library signup")]
    WrongFormatNumOfBooks,
    #[error("Wrong format of book id at library signup")]
    WrongFormatBookId,
    #[error("There is a different number of books than specified, library id {library_id}")]
    WrongNumberOfBooks {
        library_id: LibraryID
    },
    #[error("There is a different number of libraries than specified")]
    WrongNumberOfLibrariesToSignUp
}


pub enum Case {
    Example,
    A,
    B,
    C,
    D,
    E,
    F
}

type LibraryID = u32;
type BookID = u32;

struct LibrarySignup {
    id: LibraryID,
    books_to_scan: Vec<BookID>
}

impl LibrarySignup {
    fn parse_from_2_lines(first_line: &str, second_line: &str) -> Result<Self, ScoringError> {


        let mut id_and_num_of_books = first_line.split_whitespace();
        let library_id = id_and_num_of_books.next()
            .ok_or(ScoringError::MissingLibraryId)?
            .parse::<LibraryID>()
            .map_err(|_| ScoringError::WrongFormatLibraryId)?;

        let num_of_books = id_and_num_of_books.next()
            .ok_or(ScoringError::MissingNumOfBooksForLibrarySignup)?
            .parse::<u32>()
            .map_err(|_| ScoringError::WrongFormatNumOfBooks)?;

        let books_to_scan = second_line.split_whitespace()
            .map(|book_id_str| book_id_str.parse::<BookID>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| ScoringError::WrongFormatBookId)?;

        if books_to_scan.len() != num_of_books as usize {
            return Err(ScoringError::WrongNumberOfBooks{ library_id})
        }

        Ok(Self { id: library_id, books_to_scan })
    }
}

struct Submission {
    libraries_to_signup: Vec<LibrarySignup>
}


pub fn score(submission: &str, case: Case) -> Result<usize, ScoringError> {
    let mut input_lines = submission.lines();
    let number_of_libraries_to_signup: u32 = input_lines.next().ok_or(ScoringError::MissingLine)?
        .parse().map_err(|_| ScoringError::ExpectedANumber)?;

    //TODO add check that 0 <= A <= L

    let mut libraries_to_signup = Vec::with_capacity(number_of_libraries_to_signup as usize);
    for mut double_line in &input_lines.chunks(2) {
        let fl = double_line.next().ok_or(ScoringError::MissingLine)?;
        let sl = double_line.next().ok_or(ScoringError::MissingLine)?;
        let library_signup = LibrarySignup::parse_from_2_lines(fl, sl)?;
        libraries_to_signup.push(library_signup);

    }

    if libraries_to_signup.len() != number_of_libraries_to_signup as usize {
        return Err(ScoringError::WrongNumberOfLibrariesToSignUp)
    }

    let submission = Submission { libraries_to_signup };

    //TODO add checks for submission (e.g. the needed books are in the library)

    unimplemented!()

}

#[cfg(test)]
mod test {
    use crate::qual2020::{LibrarySignup, ScoringError};

    #[test]
    fn parse_library_signup() {
        let library_desc = "1 5\n1 2 3 4 5";
        let lines = library_desc.lines().collect::<Vec<_>>();
        let lib = LibrarySignup::parse_from_2_lines(lines[0], lines[1]).expect("shouldn't fail");
        assert_eq!(lib.id, 1);
        assert_eq!(lib.books_to_scan, vec![1,2,3,4,5]);
    }

    #[test]
    fn test_failed_parse_library_signup() {
        let library_desc = "1 3\n1 2 3 4 5";
        let lines = library_desc.lines().collect::<Vec<_>>();
        let failed_lib = LibrarySignup::parse_from_2_lines(lines[0], lines[1]);
        assert_eq!(failed_lib.err(), Some(ScoringError::WrongNumberOfBooks{ library_id: 1}))
    }
}
