//! This file is an adapation of Martin Sosic edlib.h to get a Rust idiomatic interface
//! simpler than the one generated by bingen, but ultimately call bingen generated interface.
//! All C structrures and functions are modified by appending "Rs" to their name.
//! We also avoid pointers.

use std::os::raw::c_char;
use std::slice;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Status codes
pub const EDLIB_RS_STATUS_OK: u32 = 0;
#[allow(dead_code)]
pub const EDLIB_RS_STATUS_ERROR: u32 = 1;

///
/// Alignment methods - how should Edlib treat gaps before and after query?
///
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub enum EdlibAlignModeRs {
    ///
    /// Global method. This is the standard and default method.
    /// Useful when you want to find out how similar is first sequence to second sequence.
    ///
    EDLIB_MODE_NW,

    /// Prefix method. Similar to global method, but with a small twist - gap at query end is not penalized.
    /// What that means is that deleting elements from the end of second sequence is "free"!
    /// For example, if we had "AACT" and "AACTGGC", edit distance would be 0, because removing "GGC" from the end
    /// of second sequence is "free" and does not count into total edit distance.    
    /// This method is appropriate when you want to find out how well first sequence fits at the beginning of second sequence.
    ///
    EDLIB_MODE_SHW,

    /// Infix method. Similar as prefix method, but with one more twist - gaps at query end and start are
    /// not penalized. What that means is that deleting elements from the start and end of second sequence is "free"!
    /// For example, if we had ACT and CGACTGAC, edit distance would be 0, because removing CG from the start
    /// and GAC from the end of second sequence is "free" and does not count into total edit distance.
    /// This method is appropriate when you want to find out how well first sequence fits at any part of
    /// second sequence.
    /// For example, if your second sequence was a long text and your first sequence was a sentence from that text,
    /// but slightly scrambled, you could use this method to discover how scrambled it is and where it fits in
    /// that text.  
    /// In bioinformatics, this method is appropriate for aligning read to a sequence.
    ///
    EDLIB_MODE_HW,
}

///
/// Alignment tasks - what do you want Edlib to do?
///
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum EdlibAlignTaskRs {
    /// Find edit distance and end locations. This is the default mode.
    EDLIB_TASK_DISTANCE,
    ///  Find edit distance, end locations and start locations.    
    EDLIB_TASK_LOC,
    /// Find edit distance, end locations and start locations and alignment path.       
    EDLIB_TASK_PATH,
}

///
/// Describes cigar format.
/// see http://samtools.github.io/hts-specs/SAMv1.pdf
///see http://drive5.com/usearch/manual/cigar.html
///
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum EdlibCigarFormatRs {
    /// Match: 'M', Insertion: 'I', Deletion: 'D', Mismatch: 'M'.
    EDLIB_CIGAR_STANDARD,
    ///    Match: '=', Insertion: 'I', Deletion: 'D', Mismatch: 'X'.
    EDLIB_CIGAR_EXTENDED,
}

/// Edit operations.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum EdlibEdopRs {
    /// Match
    EDLIB_EDOP_MATCH,
    /// Insertion to target = deletion from query
    EDLIB_EDOP_INSERT,
    /// Deletion from target = insertion to query.
    EDLIB_EDOP_DELETE,
    /// Mismatch.
    EDLIB_EDOP_MISMATCH,
}

// We use c_char here to be able to cast C pointer directly
/// Defines two given characters as equal.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct EdlibEqualityPairRs {
    first: ::std::os::raw::c_char,
    second: ::std::os::raw::c_char,
}

//=================================================================================================
///
/// Configuration object for edlibAlign() function.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct EdlibAlignConfigRs<'a> {
    /// Set k to non-negative value to tell edlib that edit distance is not larger than k.
    /// Smaller k can significantly improve speed of computation.
    /// If edit distance is larger than k, edlib will set edit distance to -1.
    /// Set k to negative value and edlib will internally auto-adjust k until score is found.
    pub k: i32,

    /// Alignment method:  
    /// EDLIB_MODE_NW: global (Needleman-Wunsch).    
    /// EDLIB_MODE_SHW: prefix. Gap after query is not penalized.  
    /// EDLIB_MODE_HW: infix. Gaps before and after query are not penalized.
    ///
    pub mode: EdlibAlignModeRs,

    /// Alignment task - tells Edlib what to calculate. Less to calculate, faster it is.  
    /// EDLIB_TASK_DISTANCE - find edit distance and end locations of optimal alignment paths in target.  
    /// EDLIB_TASK_LOC - find edit distance and start and end locations of optimal alignment paths in target.  
    /// EDLIB_TASK_PATH - find edit distance, alignment path (and start and end locations of it in target).  
    ///
    pub task: EdlibAlignTaskRs,

    /// List of pairs of characters, where each pair defines two characters as equal.
    /// This way you can extend edlib's definition of equality (which is that each character is equal only
    /// to itself).
    /// This can be useful if you have some wildcard characters that should match multiple other characters,
    /// or e.g. if you want edlib to be case insensitive.
    /// Can be set to NULL if there are none.
    pub additionalequalities: &'a [EdlibEqualityPairRs],
}

impl<'a> EdlibAlignConfigRs<'a> {
    /// Helper method for easy construction of configuration object.
    /// return Configuration object filled with given parameters.
    pub fn new(
        k: i32,
        mode: EdlibAlignModeRs,
        task: EdlibAlignTaskRs,
        additionalequalities: &'a [EdlibEqualityPairRs],
    ) -> Self {
        EdlibAlignConfigRs {
            k,
            mode,
            task,
            additionalequalities,
        }
    }
}

impl<'a> Default for EdlibAlignConfigRs<'a> {
    ///      k = -1, mode = EDLIB_MODE_NW, task = EDLIB_TASK_DISTANCE, no additional equalities.
    fn default() -> Self {
        EdlibAlignConfigRs {
            k: -1,
            mode: EdlibAlignModeRs::EDLIB_MODE_NW,
            task: EdlibAlignTaskRs::EDLIB_TASK_DISTANCE,
            additionalequalities: &[],
        }
    }
}

//================================================================================================

/// Container for results of alignment done by edlibAlign() function.
#[derive(Debug, Clone)]
pub struct EdlibAlignResultRs {
    /// EDLIB_STATUS_OK or EDLIB_STATUS_ERROR. If error, all other fields will have undefined values.
    pub status: u32,

    /// -1 if k is non-negative and edit distance is larger than k.
    pub editDistance: i32,

    /// Array of zero-based positions in target where optimal alignment paths end.
    /// If gap after query is penalized, gap counts as part of query (NW), otherwise not.
    /// Set to None if edit distance is larger than k.
    pub endLocations: Option<Vec<i32>>,

    /// Array of zero-based positions in target where optimal alignment paths start,
    /// they correspond to endLocations.
    /// If gap before query is penalized, gap counts as part of query (NW), otherwise not.
    /// Set to NULL if not calculated or if edit distance is larger than k.
    pub startLocations: Option<Vec<i32>>,

    /// Number of end (and start) locations.
    pub numLocations: usize,

    /// Alignment is found for first pair of start and end locations.
    /// Set to NULL if not calculated.
    /// Alignment is sequence of numbers: 0, 1, 2, 3.
    ///         0 stands for match.  
    ///         1 stands for insertion to target.  
    ///         2 stands for insertion to query.  
    ///         3 stands for mismatch.  
    /// Alignment aligns query to target from begining of query till end of query.
    /// If gaps are not penalized, they are not in alignment.
    pub alignment: Option<Vec<u8>>,

    /// Number of different characters in query and target together.
    pub alphabetLength: u32,
} // end of struct EdlibAlignResultRs

impl EdlibAlignResultRs {
    /// get result distance
    pub fn getDistance(&self) -> i32 {
        return self.editDistance;
    }
    /// get end locations of optimal alignment path
    pub fn getEndLocations(&self) -> Option<&Vec<i32>> {
        return self.endLocations.as_ref();
    }
    /// get start locations of optimal alignment path
    pub fn getStartLocations(&self) -> Option<&Vec<i32>> {
        return self.startLocations.as_ref();
    }
    ///
    pub fn getAlignment(&self) -> Option<&Vec<u8>> {
        return self.alignment.as_ref();
    }
} // end EdlibAlignResultRs block

impl Default for EdlibAlignResultRs {
    ///   k = -1, mode = EDLIB_MODE_NW, task = EDLIB_TASK_DISTANCE, no additional equalities.
    fn default() -> Self {
        EdlibAlignResultRs {
            status: EDLIB_STATUS_OK,
            editDistance: 0,
            endLocations: None,
            startLocations: None,
            numLocations: 0,
            alignment: None,
            alphabetLength: 0,
        }
    }
} // end impl Default for EdlibAlignResultRs

/// Aligns two sequences (query and target) using edit distance (levenshtein distance).  
/// Through config parameter, this function supports different alignment methods (global, prefix, infix),
/// as well as different modes of search (tasks).  
/// It always returns edit distance and end locations of optimal alignment in target.
/// It optionally returns start locations of optimal alignment in target and alignment path,
/// if you choose appropriate tasks.  
/// Parameters:  
///     . query  : First sequence.  
///     . target : Second sequence.  
///     . config : Additional alignment parameters, like alignment method and wanted results.  
/// Result of alignment, which can contain edit distance, start and end locations and alignment path.  
/// **Note**:  
///  Rust interface causes cloning of start/end locations, ensures i32 representations of locations and so transfer
/// memory responsability to Rust.

pub fn edlibAlignRs(
    query: &[u8],
    target: &[u8],
    config_rs: &EdlibAlignConfigRs,
) -> EdlibAlignResultRs {
    // real work here
    // get pointers to query and target to EdlibEqualityPair form config
    let mut config_c = unsafe { edlibDefaultAlignConfig() };
    config_c.k = config_rs.k as ::std::os::raw::c_int;
    config_c.mode = match config_rs.mode {
        EdlibAlignModeRs::EDLIB_MODE_NW => 0 as EdlibAlignMode,
        EdlibAlignModeRs::EDLIB_MODE_SHW => 1 as EdlibAlignMode,
        EdlibAlignModeRs::EDLIB_MODE_HW => 2 as EdlibAlignMode,
    };
    config_c.task = match config_rs.task {
        EdlibAlignTaskRs::EDLIB_TASK_DISTANCE => 0 as EdlibAlignTask,
        EdlibAlignTaskRs::EDLIB_TASK_LOC => 1 as EdlibAlignTask,
        EdlibAlignTaskRs::EDLIB_TASK_PATH => 2 as EdlibAlignTask,
    };
    config_c.additionalEqualitiesLength =
        config_rs.additionalequalities.len() as ::std::os::raw::c_int;
    if config_c.additionalEqualitiesLength > 0 {
        config_c.additionalEqualities =
            config_rs.additionalequalities.as_ptr() as *const EdlibEqualityPair;
    } else {
        config_c.additionalEqualities = ::std::ptr::null::<EdlibEqualityPair>();
    }

    // Recast to EdlibAlignResultRs
    let res_c: EdlibAlignResult = unsafe {
        edlibAlign(
            query.as_ptr() as *const ::std::os::raw::c_char,
            query.len() as ::std::os::raw::c_int,
            target.as_ptr() as *const ::std::os::raw::c_char,
            target.len() as ::std::os::raw::c_int,
            // now config
            config_c,
        )
    };
    // go back to EdlibAlignResultRs. Clone incurs some cost. Should go to impl From<EdlibAlignResult>
    let mut align_res_rs = EdlibAlignResultRs::default();
    align_res_rs.status = res_c.status as u32;
    align_res_rs.editDistance = res_c.editDistance as i32;
    align_res_rs.numLocations = res_c.numLocations as usize;
    // get  ::std::os::raw::c_int slices for endLocations
    if res_c.numLocations > 0 {
        assert!(res_c.endLocations != std::ptr::null_mut());
        let s_end =
            unsafe { slice::from_raw_parts(res_c.endLocations, res_c.numLocations as usize) };
        assert_eq!(s_end.len(), align_res_rs.numLocations);
        align_res_rs.endLocations = Some(s_end.to_vec());
        // we have startLocations only if task == LOC or PATH so we must check
        if res_c.startLocations != std::ptr::null_mut() {
            let s_start: &[::std::os::raw::c_int] =
                unsafe { slice::from_raw_parts(res_c.startLocations, res_c.numLocations as usize) };
            assert_eq!(s_start.len(), align_res_rs.numLocations);
            align_res_rs.startLocations = Some(s_start.to_vec());
        }
    }
    if res_c.alignmentLength > 0 {
        assert!(
            res_c.alignment != std::ptr::null_mut(),
            "null alignment pointer"
        );
        let s_align =
            unsafe { slice::from_raw_parts(res_c.alignment, res_c.alignmentLength as usize) };
        align_res_rs.alignment = Some(s_align.to_vec());
    }
    align_res_rs.alphabetLength = res_c.alphabetLength as u32;
    // Free C datas
    unsafe {
        edlibFreeAlignResult(res_c);
    };
    //
    align_res_rs
}

extern "C" {
    fn free(s: *const c_char);
}

/// Builds cigar string from given alignment sequence.  
///  param : alignment  Alignment sequence.
///  (is obtained from EdlibAlignResultRs.alignment which is a Some if EdlibAlignConfigRs.task is set to EdlibAlignTaskRs::EDLIB_TASK_PATH
///  see *test_path_hw*)
///     *  0 stands for match.
///     *  1 stands for insertion to target.
///     *  2 stands for insertion to query.
///     *  3 stands for mismatch.
///  param cigarFormat  Cigar will be returned in specified format.
///
///   return Cigar string where :
///    * I stands for insertion.  
///    * D stands for deletion.  
///    * X stands for mismatch. (used only in extended format)
///    * = stands for match. (used only in extended format)
///    * M stands for (mis)match. (used only in standard format)

pub fn edlibAlignmentToCigarRs(alignment: &[u8], cigarFormat: &EdlibCigarFormatRs) -> String {
    // convert cigarFormat to C arg
    let cigarstring: String;
    unsafe {
        let c_res: *const c_char = edlibAlignmentToCigar(
            alignment.as_ptr(),
            alignment.len() as i32,
            *cigarFormat as u32,
        );
        assert!(
            c_res != std::ptr::null_mut(),
            "null cigar string returnd from C"
        );
        cigarstring = ::std::ffi::CStr::from_ptr(c_res)
            .to_string_lossy()
            .into_owned();
        free(c_res);
    }
    cigarstring
}

//===================================================================

#[cfg(test)]

mod tests {

    use super::*;

    #[test]
    fn test_distance_nw() {
        let query = "ACCTCTG";
        let target = "ACTCTGAAA";
        let align_res = edlibAlignRs(
            query.as_bytes(),
            target.as_bytes(),
            &EdlibAlignConfigRs::default(),
        );
        assert_eq!(align_res.status, EDLIB_STATUS_OK);
        assert_eq!(align_res.getDistance(), 4);
    } // end test_distance_nw

    #[test]
    fn test_distance_shw() {
        let query = "ACCTCTG";
        let target = "ACTCTGAAA";
        //
        let mut config = EdlibAlignConfigRs::default();
        config.mode = EdlibAlignModeRs::EDLIB_MODE_SHW;
        let align_res = edlibAlignRs(query.as_bytes(), target.as_bytes(), &config);
        assert_eq!(align_res.status, EDLIB_STATUS_OK);
        assert_eq!(align_res.editDistance, 1);
    } // end test_distance_shw

    #[test]
    fn test_distance_hw() {
        let query = "ACCTCTG";
        let target = "TTTTTTTTTTTTTTTTTTTTTACTCTGAAA";
        //
        let mut config = EdlibAlignConfigRs::default();
        config.mode = EdlibAlignModeRs::EDLIB_MODE_HW;
        let align_res = edlibAlignRs(query.as_bytes(), target.as_bytes(), &config);
        assert_eq!(align_res.status, EDLIB_STATUS_OK);
        assert_eq!(align_res.editDistance, 1);
    } // end test_distance_hw

    #[test]
    fn test_distance_hw_with_pair() {
        let query = "ACCTCTG";
        let target = "TTTTTTTTTTTTTTTTTTTTTNCTCTXAAA";
        let mut equalitypairs = Vec::<EdlibEqualityPairRs>::new();
        let pair = EdlibEqualityPairRs {
            first: 'A' as c_char,
            second: 'N' as c_char,
        };
        equalitypairs.push(pair);
        let pair = EdlibEqualityPairRs {
            first: 'G' as c_char,
            second: 'X' as c_char,
        };
        equalitypairs.push(pair);
        let mut config = EdlibAlignConfigRs::default();
        config.mode = EdlibAlignModeRs::EDLIB_MODE_HW;
        config.additionalequalities = &equalitypairs;
        let align_res = edlibAlignRs(query.as_bytes(), target.as_bytes(), &config);
        assert_eq!(align_res.status, EDLIB_STATUS_OK);
        assert_eq!(align_res.editDistance, 1);
    } // end of test_distance_with_pair

    #[test]
    fn test_path_hw() {
        let query = "missing";
        let target = "mississipi";
        //
        let mut config = EdlibAlignConfigRs::default();
        config.mode = EdlibAlignModeRs::EDLIB_MODE_HW;
        config.task = EdlibAlignTaskRs::EDLIB_TASK_PATH;
        let align_res = edlibAlignRs(query.as_bytes(), target.as_bytes(), &config);
        assert_eq!(align_res.status, EDLIB_STATUS_OK);
        assert_eq!(align_res.editDistance, 2);
        assert!(align_res.startLocations.is_some());
        assert!(align_res.endLocations.is_some());
        //
        assert!(align_res.getAlignment().is_some());

        let cigar = edlibAlignmentToCigarRs(
            align_res.alignment.as_ref().unwrap(),
            &EdlibCigarFormatRs::EDLIB_CIGAR_STANDARD,
        );
        // answer is "5M2I"
        println!(" cigar : {:?}", cigar);
        assert_eq!(cigar, "5M2I");

        let cigarx = edlibAlignmentToCigarRs(
            align_res.getAlignment().unwrap(),
            &EdlibCigarFormatRs::EDLIB_CIGAR_EXTENDED,
        );
        // answer is "5=2I"
        println!(" cigar : {:?}", cigarx);
        assert_eq!(cigarx, "5=2I");
    } // end of test_path_hw

    #[test]
    fn test_distance_nw_with_max_k() {
        let query = "ACCTCTG";
        let target = "ACTCTGAAA";
        let mut config = EdlibAlignConfigRs::default();
        config.k = 3;
        let align_res = edlibAlignRs(query.as_bytes(), target.as_bytes(), &config);
        assert_eq!(align_res.status, EDLIB_STATUS_OK);
        // real distance is 4 as we asked for max dist = 3 we should get -1
        assert_eq!(align_res.getDistance(), -1);
    } // end test_distance_nw
} // mod tests
