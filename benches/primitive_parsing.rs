#![feature(test)]

extern crate test;

use test::Bencher;

use pulldown_latex::parser::Parser;

#[bench]
fn match_on_greek(b: &mut Bencher) {
    b.iter(|| {
    let storage = pulldown_latex::parser::Storage::new();
    let parser = Parser::new(r"
\alpha \beta \gamma \delta \epsilon \zeta \eta \theta
\iota \kappa \lambda \mu \nu \xi \omicron \pi
\rho \sigma \tau \upsilon \phi \chi \psi \omega
\Alpha \Beta \Gamma \Delta \Epsilon \Zeta \Eta \Theta
\Iota \Kappa \Lambda \Mu \Nu \Xi \Omicron \Pi
\Rho \Sigma \Tau \Upsilon \Phi \Chi \Psi \Omega
\varGamma \varDelta \varTheta \varLambda \varXi \varPi \varSigma \varUpsilon \varPhi \varPsi \varOmega
\varepsilon \vartheta \varkappa \varrho \varsigma \varpi \digamma \varphi
\alpha \beta \gamma \delta \epsilon \zeta \eta \theta
\alpha \beta \gamma \delta \epsilon \zeta \eta \theta
\iota \kappa \lambda \mu \nu \xi \omicron \pi
\rho \sigma \tau \upsilon \phi \chi \psi \omega
\Alpha \Beta \Gamma \Delta \Epsilon \Zeta \Eta \Theta
\Iota \Kappa \Lambda \Mu \Nu \Xi \Omicron \Pi
\Rho \Sigma \Tau \Upsilon \Phi \Chi \Psi \Omega
\varGamma \varDelta \varTheta \varLambda \varXi \varPi \varSigma \varUpsilon \varPhi \varPsi \varOmega
\varepsilon \vartheta \varkappa \varrho \varsigma \varpi \digamma \varphi
\iota \kappa \lambda \mu \nu \xi \omicron \pi
\rho \sigma \tau \upsilon \phi \chi \psi \omega
\Alpha \Beta \Gamma \Delta \Epsilon \Zeta \Eta \Theta
\Iota \Kappa \Lambda \Mu \Nu \Xi \Omicron \Pi
", &storage);
        let mut str = String::new();
        test::black_box(pulldown_latex::mathml::push_mathml(&mut str, parser, Default::default()).unwrap());
    });
}

#[bench]
fn subscript_torture(b: &mut Bencher) {
    b.iter(|| {
        let storage = pulldown_latex::parser::Storage::new();
        let parser = Parser::new("a_{5_{5_{5_{5_{5_{5_{5_{5_{5_{5_{5_5}}}}}}}}}}}", &storage);
        let mut str = String::new();
        test::black_box(
            pulldown_latex::mathml::push_mathml(&mut str, parser, Default::default()).unwrap(),
        );
    });
}

#[bench]
fn basic_macros(b: &mut Bencher) {
    b.iter(|| {
        let storage = pulldown_latex::parser::Storage::new();
        let parser = Parser::new(
            r"\def\d{\mathrm{d}}
                \oint_C \vec{B}\circ \d\vec{l} = \mu_0 \left( I_{\text{enc}}
                + \varepsilon_0 \frac{\d}{\d t} \int_S {\vec{E} \circ \hat{n}}\;
                \d a \right)",
            &storage,
        );
        let mut str = String::new();
        test::black_box(
            pulldown_latex::mathml::push_mathml(&mut str, parser, Default::default()).unwrap(),
        );
    });
}
