//! Crate documentation supplements
//! 
//! <script defer>
//! setTimeout(() => {
//!     // after whole document loaded
//! 
//!     let heading = document.querySelector("section#main-content .main-heading");
//!     heading.style.display = "flex";
//!     heading.style.flexDirection = "column";
//!     
//!     let pathChain = document.createElement("span");
//!     pathChain.classList.add("out-of-band");
//!     pathChain.style.display = "flex";
//!     let separator = document.createElement("span");
//!     separator.innerText = "::";
//!     for (const item of [...heading.querySelectorAll("h1 a")].slice(0, -1)) {
//!         let it = document.createElement("a");
//!         it.href = item.href;
//!         it.innerText = item.textContent;
//!         it.style.color = "var(--main-color)";
//!     
//!         pathChain.appendChild(it);
//!         pathChain.appendChild(separator.cloneNode(true));
//!     }
//!     
//!     heading.innerHTML="";
//!     heading.appendChild(pathChain);
//!     const titleEl = document.createElement("h1");
//!     titleEl.innerText = "Documentation";
//!     heading.appendChild(titleEl);
//!     
//!     // Remove top documentation section
//!     const topDoc = document.querySelector("details.top-doc");
//!     topDoc.firstElementChild.remove();
//!     topDoc.querySelector("p").remove(); // remove first paragraph (title)
//!     const forceDoc = document.createElement("div");
//!     forceDoc.id = "documentation";
//!     forceDoc.innerHTML = topDoc.innerHTML;
//!     topDoc.replaceWith(forceDoc);
//!     
//!     // Remove Reexports
//!     const reexports = document.querySelector("#reexports");
//!     reexports.nextSibling.remove();
//!     reexports.remove();
//!     
//!     // Repurpose Modules as TOC
//!     const modules = document.querySelector("#modules");
//!     modules.id = "toc";
//!     modules.innerText = "Table of Contents";
//!     
//!     const tocPrev = modules.nextSibling;
//!     const tocHtml = tocPrev.innerHTML;
//!     const toc = document.createElement("ol");
//!     toc.style.listStyleType = "upper-roman";
//!     
//!     const items = tocPrev.querySelectorAll("li");
//!     for (const item of items) {
//!         let linkSource = item.querySelector(".item-name>a");
//!         let link = document.createElement("a");
//!         link.href = linkSource.href;
//!         link.innerText = linkSource.textContent;
//!         let descNode = item.querySelector(".desc");
//!         if (descNode != null) {
//!             link.innerText = descNode.textContent;
//!         }
//!         let result = document.createElement("li");
//!         result.appendChild(link);
//!         toc.appendChild(result);
//!     }
//!     
//!     tocPrev.replaceWith(toc);
//!     
//!     // Cleanup sidebar elements
//!     document.querySelector("nav.sidebar>h2.location>a").innerText = "Documentation";
//!     let oldLinks = document.querySelector("nav.sidebar .sidebar-elems section>ul.block");
//!     oldLinks.replaceWith(toc.cloneNode(true));
//! });
//! </script>

/// Stylesheet that adds simple clip-path based icons.
/// 
/// They're used like so:
/// ```html
/// <i role="img" arial-label="icon-name" />
/// ```
/// 
/// `icon-name`: is a meaningful description of icon meaning.
/// 
/// This satisfies ARIA requirements and looks as expected.
macro_rules! icons {
    () => { r#"<style>
:root[data-theme="light"] {
    --icon-c-ideal: #64560d;
    --icon-c-good: #0f600c;
    --icon-c-neutral: #1e4c79;
    --icon-c-bad: #a42c2c;
}
:root[data-theme="dark"] {
    --icon-c-ideal: #ffebb0;
    --icon-c-good: #86df82;
    --icon-c-neutral: #b1d9ff;
    --icon-c-bad: #ffbaba;
}
:root[data-theme="ayu"] {
    --icon-c-ideal: #f7d160;
    --icon-c-good: #66c661;
    --icon-c-neutral: #82b6e8;
    --icon-c-bad: #ea9393;
}


i[role=img] {
    display: inline-block;
    width: 1em;
    height: 1em;
    background-color: currentColor;
    vertical-align: baseline;
    margin-bottom: -0.2em;
    overflow: hidden;

    &::before {
        display: none;
    }

    &[aria-label="good"],
    &[aria-label="bad"],
    &[aria-label="best"] {
        --w: 15%;
        --side: 100%;

        --triangle-height: calc((var(--side) / 2) / tan(30deg));
        --hp: calc((100% - var(--side)) / 2);
        --y: calc(100% - var(--triangle-height));
        --y-in: calc(var(--y) + var(--w) / sin(30deg));
        --x-in: calc(var(--hp) + var(--w) / tan(30deg));
        --x-in-max: calc(100% - var(--x-in));
        --y-in-bottom: calc(100% - var(--w));
        clip-path: polygon(
            var(--hp) 100%,calc(var(--hp) + var(--side)) 100%,50% var(--y),50% var(--y-in),var(--x-in-max) var(--y-in-bottom),var(--x-in) var(--y-in-bottom),50% var(--y-in),50% var(--y),var(--hp) 100%
        );
        transform: translateY(calc(var(--y) * -1));
    }
    &[aria-label="good"] {
        color: var(--icon-c-good);
    }
    &.invert[aria-label="good"] {
        color: var(--icon-c-bad);
    }
    &[aria-label="bad"] {
        color: var(--icon-c-bad);

        rotate: 180deg;
        transform: translateY(calc(var(--y) * -1));
    }
    &.invert[aria-label="bad"] {
        color: var(--icon-c-good);
    }
    &[aria-label="best"] {
        color: var(--icon-c-ideal);

        --w: 12%;
        --top-w: 12%;
        --gap: 10%;

        --top-offset: calc(var(--top-w) / sin(30deg));
        --side: calc(100% - var(--top-offset) - var(--gap));
        --bb-y: calc(100% - var(--gap));
        --bb-x-off: calc(var(--top-offset) * tan(30deg));
        --bt-y: calc(var(--bb-y) - var(--triangle-height));
        --tb-y: calc(var(--bb-y) - var(--top-offset));
        --tt-y: calc(var(--bt-y) - var(--top-offset));
        clip-path: polygon(
            var(--hp) 100%,calc(var(--hp) + var(--side)) 100%,50% var(--y),50% var(--y-in),var(--x-in-max) var(--y-in-bottom),var(--x-in) var(--y-in-bottom),50% var(--y-in),50% var(--y),var(--hp) 100%,
            calc(var(--hp) + var(--bb-x-off)) var(--tb-y),50% var(--bt-y),calc(100% - var(--hp) - var(--bb-x-off)) var(--tb-y),calc(100% - var(--hp)) var(--tb-y),50% var(--tt-y),var(--hp) var(--tb-y),calc(var(--hp) + var(--bb-x-off)) var(--tb-y)
        );
        transform: initial;
    }

    &[aria-label="ok"] {
        color: var(--icon-c-neutral);

        --w: 15%;
        --inset: calc(var(--w) * sqrt(2));
        clip-path: polygon(
            50% 0,100% 50%,50% 100%,0 50%,50% 0,
            50% var(--inset),var(--inset) 50%,50% calc(100% - var(--inset)),calc(100% - var(--inset)) 50%,50% var(--inset)
        );
    }

    &[aria-label="yes"],
    &[aria-label="no"] {
        --w-diag: 10.6%;
    }
    &[aria-label="yes"] {
        color: var(--icon-c-good);

        clip-path: polygon(33% 90%,99% 24%,calc(99% - var(--w-diag)) calc(24% - var(--w-diag)),33% calc(57% + var(--w-diag)),var(--w-diag) calc(57% - var(--w-diag)),0 57%);
    }
    &.invert[aria-label="yes"] {
        color: var(--icon-c-bad);
    }
    &[aria-label="no"] {
        color: var(--icon-c-bad);

        clip-path: polygon(
            var(--w-diag) 0,0 var(--w-diag),calc(50% - var(--w-diag)) 50%,0 calc(100% - var(--w-diag)),var(--w-diag) 100%,50% calc(50% + var(--w-diag)),
            calc(100% - var(--w-diag)) 100%,100% calc(100% - var(--w-diag)),calc(50% + var(--w-diag)) 50%,100% var(--w-diag),calc(100% - var(--w-diag)) 0,50% calc(50% - var(--w-diag))
        );
        transform: translateY(calc(var(--w-diag) * -1));
    }
    &.invert[aria-label="no"] {
        color: var(--icon-c-good);
    }
}
</style>"#
    }
}

/// Generates glue that:
/// - Shows the correct (provided) title in TOC
/// - Cleans up page display
macro_rules! doc_item {
    ($title: literal) => { concat![
        "# ", $title, "\n\n",
        r#"<script defer>
let heading = document.querySelector("section#main-content .main-heading");
heading.style.display = "flex";
heading.style.flexDirection = "column";

let pathChain = document.createElement("span");
pathChain.classList.add("out-of-band");
pathChain.style.display = "flex";
let separator = document.createElement("span");
separator.innerText = "::";
for (const item of [...heading.querySelectorAll("h1 a")].slice(0, -2)) {
    let it = document.createElement("a");
    it.href = item.href;
    it.innerText = item.textContent;
    it.style.color = "var(--main-color)";

    pathChain.appendChild(it);
    pathChain.appendChild(separator.cloneNode(true));
}
let backTOC = document.createElement("a");
backTOC.href = "../index.html";
backTOC.innerText = "documentation";
pathChain.appendChild(backTOC);
pathChain.appendChild(separator);

heading.innerHTML="";
heading.appendChild(pathChain);
const titleEl = document.createElement("h1");
titleEl.innerText = ""#, $title, r#"";
heading.appendChild(titleEl);

const sidebar = document.querySelector("nav.sidebar");
const sidebarLocation = sidebar.querySelector("h2.location>a");
sidebarLocation.innerText = ""#, $title, r#"";

const sidebarElements = sidebar.querySelector(".sidebar-elems");
const inMarker = sidebarElements.querySelector("h2>a");
inMarker.innerText = "In supplemental documentation";

setTimeout(() => {
    const topDoc = document.querySelector("details.top-doc");

    // Remove top documentation section used for TOC generation
    const content = topDoc.querySelector(".docblock");
    content.firstElementChild.remove(); // usually heading

    // Prevent hiding the entire doc page
    topDoc.firstElementChild.remove(); // summary
    const inner = topDoc.firstElementChild;
    inner.remove();
    topDoc.parentElement.appendChild(inner);
    topDoc.remove();
});

setTimeout(function retry() {
    let modules = sidebarElements.querySelectorAll("h3>a");
    modules = Array.from(modules).find((it) => it.href.endsWith('#modules'));
    if (modules) {
        modules.innerText = "Table of Contents";
    } else {
        setTimeout(retry, 10);
    }
    // fixing TOC names not practical without proc_macro.
});
</script>"#]}
}

pub mod _01_comparison_table {
    #![doc = doc_item!("Comparison Table")]
    //! <div style="width:100%;display:flex;gap:1rem;overflow-x:auto">
    //! <div>
    //!
    //! | **Property** | [`BucketBackend`] | [`StringBackend`] | [`BufferBackend`] |
    //! |:-----------------------------------------------------|:--:|:--:|:--:|
    //! | [**Insertion**](#insertion)                   | <i role="img" aria-label="ok"></i>   | <i role="img" aria-label="good"></i> | <i role="img" aria-label="best"></i> |
    //! | [**Resolution**](#resolution)                 | <i role="img" aria-label="best"></i> | <i role="img" aria-label="good"></i> | <i role="img" aria-label="bad"></i>  |
    //! | [**Allocations**](#allocations)               | <i role="img" aria-label="ok"></i>   | <i role="img" aria-label="good"></i> | <i role="img" aria-label="best"></i> |
    //! | [**Memory footprint**](#memory-footprint)     | <i role="img" aria-label="bad"></i>  | <i role="img" aria-label="good"></i> | <i role="img" aria-label="best"></i> |
    //! | [**Iteration**](#iteration)                   | <i role="img" aria-label="best"></i> | <i role="img" aria-label="good"></i> | <i role="img" aria-label="bad"></i>  |
    //! | [**Contiguous**](#contiguous)                 | <i role="img" aria-label="yes"></i>  | <i role="img" aria-label="yes"></i>  | <i role="img" aria-label="no"></i>   |
    //! | [**Intern `'static`**](#intern-static)        | <i role="img" aria-label="yes"></i>  | <i role="img" aria-label="no"></i>   | <i role="img" aria-label="no"></i>   |
    //! | [**Concurrent symbols**](#concurrent-symbols) | <i role="img" aria-label="yes"></i>  | <i role="img" aria-label="yes"></i>  | <i role="img" aria-label="yes"></i>  |
    //! | [**Concurrent storage**](#concurrent-storage) | <i role="img" aria-label="yes"></i>  | <i role="img" aria-label="no"></i>  | <i role="img" aria-label="no"></i>  |
    //! 
    //! </div><div style="min-width:max-content">
    //! 
    //! #### Legend
    //! 
    //! <ul style="list-style-type:none;padding-left:0">
    //!   <li><i role="img" aria-label="best"></i> Best</li>
    //!   <li><i role="img" aria-label="good"></i> Good</li>
    //!   <li><i role="img" aria-label="ok"></i> Ok</li>
    //!   <li><i role="img" aria-label="bad"></i> Bad</li>
    //! </ul>
    //! <ul style="list-style-type:none;padding-left:0">
    //!   <li><i role="img" aria-label="yes" ></i> Yes</li>
    //!   <li><i role="img" aria-label="no" ></i> No</li>
    //! </ul>
    //! </div>
    //! </div>
    //! 
    //! <style>
    //! </style>
    //! 
    //! #### Properties
    //! 
    //! ##### Insertion
    //! 
    //! Efficiency of interning new strings.
    //! 
    //! This metric is based on benchmarking results.
    //! 
    //! ##### Resolution
    //! 
    //! Efficiency of resolving a symbol into a string reference.
    //! 
    //! ##### Allocations
    //! 
    //! The number of (re-)allocations performed by the backend.
    //! 
    //! ##### Memory footprint
    //! 
    //! Heap memory consumtion characteristics for the backend.
    //! 
    //! ##### Iteration
    //! 
    //! Efficiency of iterating over the interned strings.
    //! 
    //! ##### Contiguous
    //! 
    //! True if the interned symbols are contiguously stored in memory.
    //! 
    //! ##### Intern `'static`
    //! 
    //! True if interner can resolve symbols to statically allocated strings
    //! that have been inserted using [`StringInterner::get_or_intern_static`].
    //! 
    //! ##### Concurrent symbols
    //! 
    //! True if returned symbols are [`Send`] + [`Sync`].
    //! 
    //! ##### Concurrent storage
    //! 
    //! True if interned strings are [`Send`] + [`Sync`] while the interner is
    //! kept alive. This means resolved strings won't be moved until the
    //! interner is dropped.
    #![doc = icons!()]
    
    use crate::interner::*;
    use crate::backend::*;
    use crate::symbol::*;
    use std::marker::*;
}
pub use _01_comparison_table as comparison_table;
