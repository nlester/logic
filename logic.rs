#![feature(box_syntax, box_patterns, custom_derive)]

// Step 1: remove implication (A -> B ~> ~A v B), (A <-> B ~> A -> B ^ B -> A)
// Step 2: use double-negation (~~F ~> F) and de morgan to push negation down to leaves
// Step 3: Repeatedly use distributive laws ('and other laws'?!?) to obtain a normal form

#[allow(unused_imports)]
use std::fmt::{self, Formatter, Display};

#[derive(Clone)]
enum Formula {
    Atom(char),
    Not(Box<Formula>),
    Implies { l: Box<Formula>, r: Box<Formula> },
    Iff { l: Box<Formula>, r: Box<Formula> },
    And(Vec<Formula>),
    Or(Vec<Formula>),
}

impl fmt::Display for Formula {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", print_formula(self))
    }
}

fn print_formula(f : &Formula) -> String {
    match f
    {
        &Formula::Atom(ref c) => format!("{}", c),
        &Formula::Not(ref n) => format!("~({})", print_formula(n.as_ref())),
        &Formula::Implies { ref l, ref r } => format!("{} -> {}", print_formula(l.as_ref()), print_formula(r.as_ref())),
        &Formula::Iff { ref l, ref r } => format!("{} <-> {}", print_formula(l.as_ref()), print_formula(r.as_ref())),
        &Formula::And(ref v) => format!("({})", v.iter().map(|ref x| print_formula(&x)).collect::<Vec<String>>().join(" AND ")),
        &Formula::Or(ref v) => format!("({})", v.iter().map(|ref x| print_formula(&x)).collect::<Vec<String>>().join(" OR ")),
    }
}

fn simplify(f : Formula) -> Formula
{
	let (sf, _) = simplify3(simplify2(simplify1(f)));
	sf
}

fn simplify1(f : Formula) -> Formula
{
    match f
    {
        g @ Formula::Atom(_) => g,
        Formula::Not(n) => Formula::Not(box simplify1(*n)),
        Formula::Implies { l, r } => Formula::Or(vec!(Formula::Not(box simplify1(*l)), simplify1(*r))),
        Formula::Iff { l, r } => {
            let ls = simplify1(*l);
            let rs = simplify1(*r);
            let nl = Formula::Or(vec!(Formula::Not(box ls.clone()), rs.clone()));
            let nr = Formula::Or(vec!(Formula::Not(box rs), ls));
            Formula::And(vec!(nl, nr))
        },
        Formula::And(v) => Formula::And(v.into_iter().map(|x| simplify1(x)).collect()),
        Formula::Or(v) => Formula::Or(v.into_iter().map(|x| simplify1(x)).collect()),
    }
}

fn simplify2(f : Formula) -> Formula
{
	match f
	{
        g @ Formula::Atom(_) => g,

        // Remove double-negation.
        Formula::Not(box Formula::Not(nn)) => simplify2(*nn),

        // Use De Morgan's laws to push down not.  Note that we have to resimplify the new not expression 
        // after this (they may, for example, form a new double-negation).
        Formula::Not(box Formula::And(v)) => Formula::Or(v.into_iter().map(|x| simplify2(Formula::Not(box x))).collect()),
        Formula::Not(box Formula::Or(v)) => Formula::And(v.into_iter().map(|x| simplify2(Formula::Not(box x))).collect()),

        g @ Formula::Not(_) => g,
        Formula::And(v) => Formula::And(v.into_iter().map(|x| simplify2(x)).collect()),
        Formula::Or(v) => Formula::Or(v.into_iter().map(|x| simplify2(x)).collect()),
        Formula::Implies { l: _, r: _ } | Formula::Iff { l: _, r: _ } => unimplemented!(),
	}
}

#[allow(dead_code)]
#[allow(unused_variables)]
fn simplify3(f: Formula) -> (Formula, bool)
{
	match f
	{
        Formula::Implies { l: _, r: _ } | Formula::Iff { l: _, r: _ } => unimplemented!(),
		g @ Formula::Atom(_) => (g, false),
		Formula::Not(nn) => 
		{
			let (snn, simplified) = simplify3(*nn);
			(Formula::Not(box snn), simplified)
		}

		Formula::Or(v) =>
		{
			let mut simplified = false;
			(Formula::Or(v.into_iter().map(|x| { let (sx, s) = simplify3(x); simplified = simplified | s; sx }).collect()), simplified)
		}

        Formula::And(v) => 
        {
        	// In CNJ, P ^ (Q v S) => (P ^ Q) v (P ^ S).

        	// Separate items into disjunctions and others (singles).
        	let mut simplified = false;
        	let mut singles = Vec::<Formula>::new();
        	let mut multiples = Vec::<Vec<Formula>>::new();
        	for el in v.into_iter().map(|x| { let (sx, s) = simplify3(x); simplified = simplified | s; sx }) 
        	{
        		match el 
        		{
        			Formula::Or(ov) => { multiples.push(ov); }
        			g @ _ => { singles.push(g); }
        		}
        	}
            
            let mut disj = Vec::<Formula>::new();
        	let iterations = multiples.iter().fold(1, |acc, ref x| acc * x.len());
        	for i in 0..iterations 
        	{
            	let mut conj : Vec<Formula> = singles.iter().cloned().collect();
        		let mut offset = i;
        		for ov in &multiples
        		{
        			let pick = offset % ov.len();
        			offset = offset / ov.len();
        			conj.push(ov[pick].clone());
        		}

        		disj.push(Formula::And(conj));
        	}

        	(Formula::Or(disj), simplified || multiples.len() > 0)
        }
	}
}

fn main() {
    let nn = Formula::Not(box Formula::Not(box Formula::Atom('A')));
    println!("{} simplifies to {}", nn, simplify(nn.clone()));

    let example = Formula::Implies { l: box Formula::And(vec!(Formula::Atom('P'), Formula::Not(box Formula::Atom('Q')))), r: box Formula::Atom('R') };
    println!("{} simplifies to {} and then to {}", example, simplify2(simplify1(example.clone())), simplify(example.clone()));

    let another = Formula::Iff { l: box Formula::Or(vec!(Formula::Atom('P'), Formula::Atom('Q'))), r: box Formula::Atom('R') };
    println!("{} simplifies to {} and then to {}", another, simplify2(simplify1(another.clone())), simplify(another.clone()));
}
