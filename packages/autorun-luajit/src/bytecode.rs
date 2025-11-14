mod writer;

/**
/* Bytecode instruction definition. Order matters, see below.
**
** (name, filler, Amode, Bmode, Cmode or Dmode, metamethod)
**
** The opcode name suffixes specify the type for RB/RC or RD:
** V = variable slot
** S = string const
** N = number const
** P = primitive type (~itype)
** B = unsigned byte literal
** M = multiple args/results
*/
#define BCDEF(_) \
  /* Comparison ops. ORDER OPR. */ \
  _(ISLT,	var,	___,	var,	lt) \
  _(ISGE,	var,	___,	var,	lt) \
  _(ISLE,	var,	___,	var,	le) \
  _(ISGT,	var,	___,	var,	le) \
  \
  _(ISEQV,	var,	___,	var,	eq) \
  _(ISNEV,	var,	___,	var,	eq) \
  _(ISEQS,	var,	___,	str,	eq) \
  _(ISNES,	var,	___,	str,	eq) \
  _(ISEQN,	var,	___,	num,	eq) \
  _(ISNEN,	var,	___,	num,	eq) \
  _(ISEQP,	var,	___,	pri,	eq) \
  _(ISNEP,	var,	___,	pri,	eq) \
  \
  /* Unary test and copy ops. */ \
  _(ISTC,	dst,	___,	var,	___) \
  _(ISFC,	dst,	___,	var,	___) \
  _(IST,	___,	___,	var,	___) \
  _(ISF,	___,	___,	var,	___) \
  _(ISTYPE,	var,	___,	lit,	___) \
  _(ISNUM,	var,	___,	lit,	___) \
  \
  /* Unary ops. */ \
  _(MOV,	dst,	___,	var,	___) \
  _(NOT,	dst,	___,	var,	___) \
  _(UNM,	dst,	___,	var,	unm) \
  _(LEN,	dst,	___,	var,	len) \
  \
  /* Binary ops. ORDER OPR. VV last, POW must be next. */ \
  _(ADDVN,	dst,	var,	num,	add) \
  _(SUBVN,	dst,	var,	num,	sub) \
  _(MULVN,	dst,	var,	num,	mul) \
  _(DIVVN,	dst,	var,	num,	div) \
  _(MODVN,	dst,	var,	num,	mod) \
  \
  _(ADDNV,	dst,	var,	num,	add) \
  _(SUBNV,	dst,	var,	num,	sub) \
  _(MULNV,	dst,	var,	num,	mul) \
  _(DIVNV,	dst,	var,	num,	div) \
  _(MODNV,	dst,	var,	num,	mod) \
  \
  _(ADDVV,	dst,	var,	var,	add) \
  _(SUBVV,	dst,	var,	var,	sub) \
  _(MULVV,	dst,	var,	var,	mul) \
  _(DIVVV,	dst,	var,	var,	div) \
  _(MODVV,	dst,	var,	var,	mod) \
  \
  _(POW,	dst,	var,	var,	pow) \
  _(CAT,	dst,	rbase,	rbase,	concat) \
  \
  /* Constant ops. */ \
  _(KSTR,	dst,	___,	str,	___) \
  _(KCDATA,	dst,	___,	cdata,	___) \
  _(KSHORT,	dst,	___,	lits,	___) \
  _(KNUM,	dst,	___,	num,	___) \
  _(KPRI,	dst,	___,	pri,	___) \
  _(KNIL,	base,	___,	base,	___) \
  \
  /* Upvalue and function ops. */ \
  _(UGET,	dst,	___,	uv,	___) \
  _(USETV,	uv,	___,	var,	___) \
  _(USETS,	uv,	___,	str,	___) \
  _(USETN,	uv,	___,	num,	___) \
  _(USETP,	uv,	___,	pri,	___) \
  _(UCLO,	rbase,	___,	jump,	___) \
  _(FNEW,	dst,	___,	func,	gc) \
  \
  /* Table ops. */ \
  _(TNEW,	dst,	___,	lit,	gc) \
  _(TDUP,	dst,	___,	tab,	gc) \
  _(GGET,	dst,	___,	str,	index) \
  _(GSET,	var,	___,	str,	newindex) \
  _(TGETV,	dst,	var,	var,	index) \
  _(TGETS,	dst,	var,	str,	index) \
  _(TGETB,	dst,	var,	lit,	index) \
  _(TGETR,	dst,	var,	var,	index) \
  _(TSETV,	var,	var,	var,	newindex) \
  _(TSETS,	var,	var,	str,	newindex) \
  _(TSETB,	var,	var,	lit,	newindex) \
  _(TSETM,	base,	___,	num,	newindex) \
  _(TSETR,	var,	var,	var,	newindex) \
  \
  /* Calls and vararg handling. T = tail call. */ \
  _(CALLM,	base,	lit,	lit,	call) \
  _(CALL,	base,	lit,	lit,	call) \
  _(CALLMT,	base,	___,	lit,	call) \
  _(CALLT,	base,	___,	lit,	call) \
  _(ITERC,	base,	lit,	lit,	call) \
  _(ITERN,	base,	lit,	lit,	call) \
  _(VARG,	base,	lit,	lit,	___) \
  _(ISNEXT,	base,	___,	jump,	___) \
  \
  /* Returns. */ \
  _(RETM,	base,	___,	lit,	___) \
  _(RET,	rbase,	___,	lit,	___) \
  _(RET0,	rbase,	___,	lit,	___) \
  _(RET1,	rbase,	___,	lit,	___) \
  \
  /* Loops and branches. I/J = interp/JIT, I/C/L = init/call/loop. */ \
  _(FORI,	base,	___,	jump,	___) \
  _(JFORI,	base,	___,	jump,	___) \
  \
  _(FORL,	base,	___,	jump,	___) \
  _(IFORL,	base,	___,	jump,	___) \
  _(JFORL,	base,	___,	lit,	___) \
  \
  _(ITERL,	base,	___,	jump,	___) \
  _(IITERL,	base,	___,	jump,	___) \
  _(JITERL,	base,	___,	lit,	___) \
  \
  _(LOOP,	rbase,	___,	jump,	___) \
  _(ILOOP,	rbase,	___,	jump,	___) \
  _(JLOOP,	rbase,	___,	lit,	___) \
  \
  _(JMP,	rbase,	___,	jump,	___) \
  \
  /* Function headers. I/J = interp/JIT, F/V/C = fixarg/vararg/C func. */ \
  _(FUNCF,	rbase,	___,	___,	___) \
  _(IFUNCF,	rbase,	___,	___,	___) \
  _(JFUNCF,	rbase,	___,	lit,	___) \
  _(FUNCV,	rbase,	___,	___,	___) \
  _(IFUNCV,	rbase,	___,	___,	___) \
  _(JFUNCV,	rbase,	___,	lit,	___) \
  _(FUNCC,	rbase,	___,	___,	___) \
  _(FUNCCW,	rbase,	___,	___,	___)

/* Bytecode opcode numbers. */
typedef enum {
#define BCENUM(name, ma, mb, mc, mt)	BC_##name,
BCDEF(BCENUM)
#undef BCENUM
  BC__MAX
} BCOp;
*/

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
	ISLT,
	ISGE,
	ISLE,
	ISGT,
	ISEQV,
	ISNEV,
	ISEQS,
	ISNES,
	ISEQN,
	ISNEN,
	ISEQP,
	ISNEP,
	ISTC,
	ISFC,
	IST,
	ISF,
	ISTYPE,
	ISNUM,
	MOV,
	NOT,
	UNM,
	LEN,
	ADDVN,
	SUBVN,
	MULVN,
	DIVVN,
	MODVN,
	ADDNV,
	SUBNV,
	MULNV,
	DIVNV,
	MODNV,
	ADDVV,
	SUBVV,
	MULVV,
	DIVVV,
	MODVV,
	POW,
	CAT,
	KSTR,
	KCDATA,
	KSHORT,
	KNUM,
	KPRI,
	KNIL,
	UGET,
	USETV,
	USETS,
	USETN,
	USETP,
	UCLO,
	FNEW,
	TNEW,
	TDUP,
	GGET,
	GSET,
	TGETV,
	TGETS,
	TGETB,
	TGETR,
	TSETV,
	TSETS,
	TSETB,
	TSETM,
	TSETR,
	CALLM,
	CALL,
	CALLMT,
	CALLT,
	ITERC,
	ITERN,
	VARG,
	ISNEXT,
	RETM,
	RET,
	RET0,
	RET1,
	FORI,
	JFORI,
	FORL,
	IFORL,
	JFORL,
	ITERL,
	IITERL,
	JITERL,
	LOOP,
	ILOOP,
	JLOOP,
	JMP,
	FUNCF,
	IFUNCF,
	JFUNCF,
	FUNCV,
	IFUNCV,
	JFUNCV,
	FUNCC,
	FUNCCW,
	MAX,
}

impl TryFrom<u8> for Op {
	type Error = ();

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match value {
			x if x == Op::ISLT as u8 => Ok(Op::ISLT),
			x if x == Op::ISGE as u8 => Ok(Op::ISGE),
			x if x == Op::ISLE as u8 => Ok(Op::ISLE),
			x if x == Op::ISGT as u8 => Ok(Op::ISGT),
			x if x == Op::ISEQV as u8 => Ok(Op::ISEQV),
			x if x == Op::ISNEV as u8 => Ok(Op::ISNEV),
			x if x == Op::ISEQS as u8 => Ok(Op::ISEQS),
			x if x == Op::ISNES as u8 => Ok(Op::ISNES),
			x if x == Op::ISEQN as u8 => Ok(Op::ISEQN),
			x if x == Op::ISNEN as u8 => Ok(Op::ISNEN),
			x if x == Op::ISEQP as u8 => Ok(Op::ISEQP),
			x if x == Op::ISNEP as u8 => Ok(Op::ISNEP),
			x if x == Op::ISTC as u8 => Ok(Op::ISTC),
			x if x == Op::ISFC as u8 => Ok(Op::ISFC),
			x if x == Op::IST as u8 => Ok(Op::IST),
			x if x == Op::ISF as u8 => Ok(Op::ISF),
			x if x == Op::ISTYPE as u8 => Ok(Op::ISTYPE),
			x if x == Op::ISNUM as u8 => Ok(Op::ISNUM),
			x if x == Op::MOV as u8 => Ok(Op::MOV),
			x if x == Op::NOT as u8 => Ok(Op::NOT),
			x if x == Op::UNM as u8 => Ok(Op::UNM),
			x if x == Op::LEN as u8 => Ok(Op::LEN),
			x if x == Op::ADDVN as u8 => Ok(Op::ADDVN),
			x if x == Op::SUBVN as u8 => Ok(Op::SUBVN),
			x if x == Op::MULVN as u8 => Ok(Op::MULVN),
			x if x == Op::DIVVN as u8 => Ok(Op::DIVVN),
			x if x == Op::MODVN as u8 => Ok(Op::MODVN),
			x if x == Op::ADDNV as u8 => Ok(Op::ADDNV),
			x if x == Op::SUBNV as u8 => Ok(Op::SUBNV),
			x if x == Op::MULNV as u8 => Ok(Op::MULNV),
			x if x == Op::DIVNV as u8 => Ok(Op::DIVNV),
			x if x == Op::MODNV as u8 => Ok(Op::MODNV),
			x if x == Op::ADDVV as u8 => Ok(Op::ADDVV),
			x if x == Op::SUBVV as u8 => Ok(Op::SUBVV),
			x if x == Op::MULVV as u8 => Ok(Op::MULVV),
			x if x == Op::DIVVV as u8 => Ok(Op::DIVVV),
			x if x == Op::MODVV as u8 => Ok(Op::MODVV),
			x if x == Op::POW as u8 => Ok(Op::POW),
			x if x == Op::CAT as u8 => Ok(Op::CAT),
			x if x == Op::KSTR as u8 => Ok(Op::KSTR),
			x if x == Op::KCDATA as u8 => Ok(Op::KCDATA),
			x if x == Op::KSHORT as u8 => Ok(Op::KSHORT),
			x if x == Op::KNUM as u8 => Ok(Op::KNUM),
			x if x == Op::KPRI as u8 => Ok(Op::KPRI),
			x if x == Op::KNIL as u8 => Ok(Op::KNIL),
			x if x == Op::UGET as u8 => Ok(Op::UGET),
			x if x == Op::USETV as u8 => Ok(Op::USETV),
			x if x == Op::USETS as u8 => Ok(Op::USETS),
			x if x == Op::USETN as u8 => Ok(Op::USETN),
			x if x == Op::USETP as u8 => Ok(Op::USETP),
			x if x == Op::UCLO as u8 => Ok(Op::UCLO),
			x if x == Op::FNEW as u8 => Ok(Op::FNEW),
			x if x == Op::TNEW as u8 => Ok(Op::TNEW),
			x if x == Op::TDUP as u8 => Ok(Op::TDUP),
			x if x == Op::GGET as u8 => Ok(Op::GGET),
			x if x == Op::GSET as u8 => Ok(Op::GSET),
			x if x == Op::TGETV as u8 => Ok(Op::TGETV),
			x if x == Op::TGETS as u8 => Ok(Op::TGETS),
			x if x == Op::TGETB as u8 => Ok(Op::TGETB),
			x if x == Op::TGETR as u8 => Ok(Op::TGETR),
			x if x == Op::TSETV as u8 => Ok(Op::TSETV),
			x if x == Op::TSETS as u8 => Ok(Op::TSETS),
			x if x == Op::TSETB as u8 => Ok(Op::TSETB),
			x if x == Op::TSETM as u8 => Ok(Op::TSETM),
			x if x == Op::TSETR as u8 => Ok(Op::TSETR),
			x if x == Op::CALLM as u8 => Ok(Op::CALLM),
			x if x == Op::CALL as u8 => Ok(Op::CALL),
			x if x == Op::CALLMT as u8 => Ok(Op::CALLMT),
			x if x == Op::CALLT as u8 => Ok(Op::CALLT),
			x if x == Op::ITERC as u8 => Ok(Op::ITERC),
			x if x == Op::ITERN as u8 => Ok(Op::ITERN),
			x if x == Op::VARG as u8 => Ok(Op::VARG),
			x if x == Op::ISNEXT as u8 => Ok(Op::ISNEXT),
			x if x == Op::RETM as u8 => Ok(Op::RETM),
			x if x == Op::RET as u8 => Ok(Op::RET),
			x if x == Op::RET0 as u8 => Ok(Op::RET0),
			x if x == Op::RET1 as u8 => Ok(Op::RET1),
			x if x == Op::FORI as u8 => Ok(Op::FORI),
			x if x == Op::JFORI as u8 => Ok(Op::JFORI),
			x if x == Op::FORL as u8 => Ok(Op::FORL),
			x if x == Op::IFORL as u8 => Ok(Op::IFORL),
			x if x == Op::JFORL as u8 => Ok(Op::JFORL),
			x if x == Op::ITERL as u8 => Ok(Op::ITERL),
			x if x == Op::IITERL as u8 => Ok(Op::IITERL),
			x if x == Op::JITERL as u8 => Ok(Op::JITERL),
			x if x == Op::LOOP as u8 => Ok(Op::LOOP),
			x if x == Op::ILOOP as u8 => Ok(Op::ILOOP),
			x if x == Op::JLOOP as u8 => Ok(Op::JLOOP),
			x if x == Op::JMP as u8 => Ok(Op::JMP),
			x if x == Op::FUNCF as u8 => Ok(Op::FUNCF),
			x if x == Op::IFUNCF as u8 => Ok(Op::IFUNCF),
			x if x == Op::JFUNCF as u8 => Ok(Op::JFUNCF),
			x if x == Op::FUNCV as u8 => Ok(Op::FUNCV),
			x if x == Op::IFUNCV as u8 => Ok(Op::IFUNCV),
			x if x == Op::JFUNCV as u8 => Ok(Op::JFUNCV),
			x if x == Op::FUNCC as u8 => Ok(Op::FUNCC),
			x if x == Op::FUNCCW as u8 => Ok(Op::FUNCCW),
			_ => Err(()),
		}
	}
}

pub use writer::*;
