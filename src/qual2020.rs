use thiserror::Error;
use itertools::Itertools;
use std::cmp::min;


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
    WrongNumberOfLibrariesToSignUp,
    #[error("Missing parameter on input file")]
    MissingParameterOnInputFile,
    #[error("You're trying to register more libraries than exist in this case")]
    TooManyLibraries,
    #[error("You're trying to scan a book that doesn't exist in the library")]
    LibraryDoesntContainBook,
    #[error("You're trying to sign up a library that doesn't exist")]
    NonExistLibrary

}



type LibraryID = u32;
type BookID = u32;
type BookScore = u16;

struct Library {
    id: LibraryID,
    books: Vec<BookID>,
    max_books_per_day: u32,
    days_to_signup: u32
}

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

pub struct Case {
    number_of_different_books: u32,
    libraries: Vec<Library>,
    number_of_days: u32,
    score_per_book: Vec<BookScore>
}

impl Case {
    fn parse(input: &str) -> Result<Self, ScoringError> {
        let mut lines = input.lines();

        let mut first_line = lines.next().ok_or(ScoringError::MissingLine)?.split_whitespace();
        let number_of_different_books = first_line.next()
            .ok_or(ScoringError::MissingParameterOnInputFile)?
            .parse()
            .map_err(|_| ScoringError::ExpectedANumber)?;


        let number_of_libraries = first_line.next()
            .ok_or(ScoringError::MissingParameterOnInputFile)?
            .parse::<u32>()
            .map_err(|_| ScoringError::ExpectedANumber)?;

        let number_of_days = first_line.next()
            .ok_or(ScoringError::MissingParameterOnInputFile)?
            .parse::<u32>()
            .map_err(|_| ScoringError::ExpectedANumber)?;


        let score_per_book = lines.next().ok_or(ScoringError::MissingLine)?.split_whitespace()
            .map(|x| x.parse::<BookScore>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| ScoringError::ExpectedANumber)?;


        let mut libraries = Vec::with_capacity(number_of_libraries as usize);
        for library_id in 0..number_of_libraries {
            let mut first_line = lines.next().ok_or(ScoringError::MissingLine)?.split_whitespace();
            let number_of_books = first_line.next()
                .ok_or(ScoringError::MissingParameterOnInputFile)?
                .parse::<u32>()
                .map_err(|_| ScoringError::ExpectedANumber)?;

            let days_to_signup = first_line.next()
                .ok_or(ScoringError::MissingParameterOnInputFile)?
                .parse()
                .map_err(|_| ScoringError::ExpectedANumber)?;

            let max_books_per_day = first_line.next()
                .ok_or(ScoringError::MissingParameterOnInputFile)?
                .parse()
                .map_err(|_| ScoringError::ExpectedANumber)?;


            let books_in_library = lines.next().ok_or(ScoringError::MissingLine)?.split_whitespace()
                .map(|x| x.parse::<BookID>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| ScoringError::ExpectedANumber)?;

            libraries.push(Library{ id: library_id, days_to_signup, max_books_per_day, books: books_in_library })

        }

        Ok(Case { number_of_different_books, libraries, number_of_days, score_per_book })

    }
}

lazy_static!{
    pub static ref CASE_A: Case = Case::parse(include_str!("../assets/2020qual/inputs/a_example.txt")).
                                        unwrap();
}

pub fn score(submission: &str, case: &Case) -> Result<u64, ScoringError> {
    let mut input_lines = submission.lines();
    let number_of_libraries_to_signup: u32 = input_lines.next().ok_or(ScoringError::MissingLine)?
        .parse().map_err(|_| ScoringError::ExpectedANumber)?;

    if number_of_libraries_to_signup as usize > case.libraries.len() {
        return Err(ScoringError::TooManyLibraries)
    }

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

    for library_signup in &submission.libraries_to_signup {
        let lib_id = library_signup.id;
        let library: &Library = case.libraries.get(lib_id as usize).ok_or(ScoringError::NonExistLibrary)?;

        if library_signup.books_to_scan.iter().any(|book| !library.books.contains(book)) {
            return Err(ScoringError::LibraryDoesntContainBook)
        }
    }

    let mut score: u64 = 0;
    let mut days_left = case.number_of_days;
    for curr_signup in &submission.libraries_to_signup {
        // wait the sign up time
        let library = &case.libraries[curr_signup.id as usize];
        let days_to_signup = library.days_to_signup;
        if days_left <= days_to_signup { break };
        days_left -= days_to_signup;

        let number_of_books_able_to_scan = min(days_left/library.max_books_per_day,
                                               curr_signup.books_to_scan.len() as u32);
        score += curr_signup.books_to_scan.iter().take(number_of_books_able_to_scan as usize)
            .map(|&bid| case.score_per_book[bid as usize])
            .sum::<BookScore>() as u64;
    }

    Ok(score)
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
