use std::cell::RefCell;

type Word = usize;
type Stack = Vec<Word>;
type Input = Vec<Word>;
type Output = Option<Word>;

enum Signals {
    Kill,
    Run(Output),
}

enum UnaryOp {
    Add1,
    Sub1,
}

impl UnaryOp {
    fn exec(&self, other: Word) -> Word {
        match *self {
            Self::Add1 => other + 1,
            Self::Sub1 => other - 1,
        }
    }
}

enum BinaryOp {
    Add,
    Sub,
    Mul,
    Eq,
    Gt,
    Geq,
}

impl BinaryOp {
    fn exec(&self, inp1: Word, inp2: Word) -> Word {
        match *self {
            Self::Add => inp1 + inp2,
            Self::Sub => inp1 - inp2,
            Self::Mul => inp1 * inp2,
            Self::Eq => if inp1 == inp2 { 1 } else { 0 },
            Self::Gt => if inp1 > inp2 { 1 } else { 0 },
            Self::Geq => if inp1 >= inp2 { 1 } else { 0 },
        }
    }
}

enum Ops {
    Push(Word),
    Unary(UnaryOp),
    Binary(BinaryOp),
    Duplicate,
    Drop,
    Read,
    Write,
    Label(Word),
    Jump(Word),
    SkipIfZero,
    Reverse(Word),
    NoOp,
}

struct Program {
    ops: Vec<Ops>,
    stack: RefCell<Stack>,
    input: RefCell<Input>,
    cursor: Word,
    debug:bool,
}

impl Program {
    fn get_op(&self) -> Option<&Ops> {
        self.ops.get(self.cursor)
    }

    fn reset(&mut self) {
        self.cursor = 0;
    }

    fn right(&mut self) {
        self.cursor += 1;
    }

    fn jump(&mut self, pos: Word) {
        self.reset();
        loop {
            match self.get_op() {
                None => panic!("JUMP failed: No label found"),
                Some(op) => match *op {
                    Ops::Label(o) if o == pos => break,
                    _ => self.right(),
                },
            }
        }
    }

    fn pop_stack(&self) -> Word {
        self.stack.borrow_mut().pop().unwrap()
    }

    fn pop_input(&self) -> Option<Word> {
        self.input.borrow_mut().pop()
    }

    fn push_stack(&self, value: Word) {
        self.stack.borrow_mut().push(value)
    }

    fn _no_output() -> Signals {
        Signals::Run(None)
    }

    fn exec_step(&mut self) -> Signals {
        use Ops::*;

        let cur_op = self.get_op();

        match cur_op {
            None => Signals::Kill,
            Some(maybe_op) => match maybe_op {
                Push(val) => {
                    self.push_stack(*val);
                    Self::_no_output()
                }

                Unary(op) => {
                    let inp = self.pop_stack();
                    let res = op.exec(inp);
                    self.push_stack(res);
                    Self::_no_output()
                }

                Binary(op) => {
                    let inp1 = self.pop_stack();
                    let inp2 = self.pop_stack();
                    let res = op.exec(inp1, inp2);
                    self.push_stack(res);
                    Self::_no_output()
                }
                Read => {
                    match self.pop_input() {
                        Some(res) => {
                            self.push_stack(res);
                            Self::_no_output()
                        }
                        None => Signals::Kill
                    }
                    
                }
                Label(_) => Self::_no_output(),

                SkipIfZero => {
                    let res = self.pop_stack();
                    if res == 0 {
                        self.right()
                    }
                    self.push_stack(res);
                    Self::_no_output()
                }

                Duplicate => {
                    let dup = self.pop_stack();
                    self.push_stack(dup);
                    self.push_stack(dup);
                    Self::_no_output()
                }
                Drop => {
                    self.pop_stack();
                    Self::_no_output()
                }
                Write => Signals::Run(Some(self.pop_stack())),

                Jump(pos) => {
                    let p = *pos;
                    self.jump(p);
                    Self::_no_output()
                },
                Reverse(n) => {
                    let mut tmp: Vec<Word> = Vec::with_capacity(*n);
                    for _ in 0..*n {
                        match self.stack.borrow_mut().pop() {
                            None => return Signals::Kill,
                            Some(v) => tmp.push(v)
                        }
                    }
                    tmp.iter().for_each(|&v| self.stack.borrow_mut().push(v));
                    Self::_no_output()
                }

                NoOp => Self::_no_output(),
            },
        }
    }

    fn run(&mut self) {
        let mut step_count = -1;
        loop {
            step_count += 1;
            if self.debug {
                println!("#{} Line: {} => Stack: {:?} Input: {:?}", step_count, self.cursor, self.stack.borrow(), self.input.borrow());
            }
            
            match self.exec_step() {
                Signals::Run(Some(out)) => {
                    println!("Step {} Line {} Output {:?}", step_count, self.cursor, out);
                    self.right()
                }
                Signals::Run(None) => self.right(),
                Signals::Kill => {
                    if self.debug {
                        println!("#{} Line: {} => Got Kill", step_count, self.cursor);
                    }
                    break
                },
            }
        }
    }
}

fn main() {
    use Ops::*;

    let code = vec![
        Label(0), // function start
        Read, // Get divisor
        Duplicate, // Dup divisor
        Read, // Get num
        Label(1), // Reminder loop start
        Binary(BinaryOp::Sub), // [div, num-div]
        Duplicate, // [div, num-div, num-div]
        Reverse(3), 
        Duplicate, 
        Reverse(3),
        Binary(BinaryOp::Gt), // num-div greater than div?
        SkipIfZero, // no! -> found reminder
        Jump(2), // no reminder yet
        Drop, // clean-up
        Drop,
        Write,// show reminder
        Jump(0), // read next input pair
        Label(2), // prepare data for next reminder round
        Drop, // [num-div, div]
        Duplicate, // [num-div,div, div]
        Reverse(3), // [div, div, num-div]
        Jump(1), // got to subtraction
    ];

    let input: Input = vec![169, 19, 11, 5, 17, 7];

    let stack = Vec::<Word>::with_capacity(10);

    let mut p = Program {
        ops: code,
        stack: RefCell::new(stack),
        cursor: 0,
        input: RefCell::new(input),
        debug: false
    };

    p.run();
}
