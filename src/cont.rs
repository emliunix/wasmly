use core::fmt;

use crate::types::*;

#[derive(Debug, Clone)]
enum AdminInstr<T: Clone> {
    Trap,
    Plain(T, Box<Self>),
    Label(usize, Vec<Val>, Option<T>, Box<Self>),
    Breaking(usize, Vec<Val>, Box<Self>)
}

#[derive(Debug)]
struct Config<I: Clone> (AdminInstr<I>, Vec<Val>);

macro_rules! impl_val(
    ($fn_name:ident, $branch:ident ,$type:ty) => {
         #[inline]
         fn $fn_name(v: &mut Vec<Val>) -> $type {
             if let Val::$branch(i) = v.pop().unwrap() {
                 i
             } else {
                 panic!("type mismatch")
             }
         }
    };
);

impl_val!(val_i32, I32, i32);
impl_val!(val_f32, F32, f32);

pub struct Instance {
    pub types: Vec<Ty>,
    pub locals: Vec<Val>,
}

impl Instance {
    pub fn new() -> Self {
        Self {
            types: vec![],
            locals: vec![],
        }
    }

    pub fn run(&mut self, instrs: &Vec<Instr>) -> Vec<Val> {
        use AdminInstr::*;
        let instr = Plain(&instrs[..], Box::new(Trap));
        let mut config = Config(instr, vec![]);
        let stop_fn = |instr: &AdminInstr<_>| if let Trap = instr { true } else { false };
        while !stop_fn(&config.0) {
            // print_config(&config);
            config = self.step(config);
        }
        config.1
    }

    fn step<'a>(&mut self, config: Config<&'a [Instr]>) -> Config<&'a [Instr]> {
        let Config(instr, mut vs) = config;
        use AdminInstr::*;
        let instr: AdminInstr<&'a [Instr]> = match instr {
            Trap => panic!("unreachable"),
            Plain(es, k) => {
                match es {
                    [] => *k,
                    _ => {
                        let e = &es[0];
                        let es_next = &es[1..];
                        let k = if !es_next.is_empty() {
                            Plain(es_next, k)
                        } else {
                            *k
                        };
                        match e {
                            Instr::Unreachable => Trap,
                            &Instr::I32Const(i) => { vs.push(Val::I32(i)); k },
                            Instr::I32Add => {
                                let a = val_i32(&mut vs);
                                let b = val_i32(&mut vs);
                                vs.push(Val::I32(a + b));
                                k
                            },
                            Instr::I32Eq => {
                                let a = val_i32(&mut vs);
                                let b = val_i32(&mut vs);
                                vs.push(Val::I32(if a == b { 1 } else { 0 }));
                                k
                            },
                            &Instr::LocalTee(i) => { self.locals[i] = vs.last().unwrap().clone(); k },
                            &Instr::LocalSet(i) => { self.locals[i] = vs.pop().unwrap().clone(); k },
                            &Instr::LocalGet(i) => { vs.push(self.locals[i].clone()); k },
                            &Instr::Br(n) => {
                                let vs = std::mem::replace(&mut vs, Vec::new());
                                Breaking(n, vs, Box::new(k))
                            },
                            Instr::If(bt, es_then, es_else) => {
                                let (n_args, n_res) = {
                                    let bt = block_type(&self.types, bt);
                                    let (args_ty, res_ty) = bt.func_tys();
                                    (args_ty.len(), res_ty.len())
                                };
                                let i = val_i32(&mut vs);
                                let vs = vs.drain(..vs.len()-n_args).collect::<Vec<_>>();
                                let k = Label(n_res, vs, None, Box::new(k));
                                if i != 0 {
                                    Plain(&es_then[..], Box::new(k))
                                } else {
                                    Plain(&es_else[..], Box::new(k))
                                }
                            },
                            Instr::Loop(bt, es_loop) => {
                                let (n_args, _) = {
                                    let bt = block_type(&self.types, bt);
                                    let (args_ty, res_ty) = bt.func_tys();
                                    (args_ty.len(), res_ty.len())
                                };
                                let vs = vs.drain(..vs.len()-n_args).collect::<Vec<_>>();
                                let k = Box::new(Label(n_args, vs, Some(&es[..1]), Box::new(k)));
                                Plain(&es_loop[..], k)
                            },
                            Instr::Block(bt, es) => {
                                let (n_args, n_res) = {
                                    let bt = block_type(&self.types, bt);
                                    let (args_ty, res_ty) = bt.func_tys();
                                    (args_ty.len(), res_ty.len())
                                };
                                let vs = vs.drain(..vs.len()-n_args).collect::<Vec<_>>();
                                let k = Box::new(Label(n_res, vs, None, Box::new(k)));
                                Plain(&es[..], k)
                            },
                        }
                    }
                }
            },
            Label(_, mut vs2, _, k) => {
                vs2.append(&mut vs);
                vs = vs2;
                *k
            },
            Breaking(i, vs1, k) => match *k {
                Plain(_, k) => Breaking(i, vs1, k),
                Label(n, mut vs2, br, k) => {
                    if i == 0 {
                        vs2.extend_from_slice(&vs1[vs1.len()-n..]);
                        vs = vs2;
                        match br {
                            None => *k,
                            Some(es) => Plain(es, k)
                        }
                    } else {
                        Breaking(i - 1, vs1, k)
                    }
                }
                _ => panic!("unreachable"),
            },
        };
        Config(instr, vs)
    }
}

fn indent(mut i: usize) {
    while i > 0 {
        print!("  ");
        i -= 1;
    }
}

fn print_cont<T: Clone + fmt::Debug>(k: &AdminInstr<T>, n: usize) {
    match k {
        AdminInstr::Trap => { indent(n); println!("trap") },
        AdminInstr::Plain(es, k) => {
            indent(n);
            println!("plain {:?}", es);
            print_cont(k, n);
        },
        AdminInstr::Label(i, vs, es, k) => {
            indent(n); println!("label");
            indent(n+1); println!("vs: {:?}", vs);
            indent(n+1); println!("es: {:?}", es);
            print_cont(k, n);
        },
        AdminInstr::Breaking(i, vs, k) => {
            indent(n); println!("break {}", i);
            indent(n+1); println!("vs: {:?}", vs);
            print_cont(k, n);
        },
    }
}

fn print_config<T: Clone + fmt::Debug>(config: &Config<T>) {
    let Config(k, vs) = config;
    println!(">>> CONFIG DUMP");
    println!("value stack: {:?}", vs);
    println!("continuation:");
    print_cont(k, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    pub fn test() {
        let mut vm = Instance::new();
        let result = vm.run(&vec![
            Instr::I32Const(1),
            Instr::I32Const(1),
            Instr::I32Add,
        ]).last().cloned();
        assert_eq!(Some(Val::I32(2)), result);
    }

    #[test]
    pub fn test_block() {
        let mut vm = Instance::new();
        let result = vm.run(&vec![
            Instr::Block(
                BlockType::ValTy(Ty::I32),
                vec![
                    Instr::I32Const(1),
                    Instr::I32Const(1),
                    Instr::I32Add,
                ]),
        ]).last().cloned();
        assert_eq!(Some(Val::I32(2)), result);
    }

    #[test]
    pub fn test_loop() {
        let mut vm = Instance::new();
        vm.locals.push(Val::I32(0));
        let result = vm.run(&vec![
            Instr::Block(BlockType::Empty, vec![
                Instr::Loop(BlockType::Empty, vec![
                    Instr::LocalGet(0),
                    Instr::I32Const(1),
                    Instr::I32Add,
                    Instr::LocalTee(0),
                    Instr::I32Const(20),
                    Instr::I32Eq,
                    Instr::If(BlockType::Empty, vec![
                        Instr::Br(2),
                    ], vec![
                        Instr::Br(1),
                    ]),
                ])
            ]),
            Instr::LocalGet(0),
        ]).last().cloned();
        assert_eq!(Some(Val::I32(20)), result);
    }
}
