require "../spec_helper"

describe FsFormula do
  it "parses an empty formula as no tokens" do
    FsFormula.parse("").should eq([] of FsToken)
    FsFormula.parse("   ").should eq([] of FsToken)
  end

  it "parses closing debit and credit" do
    t = FsFormula.parse("+304Д")
    t.size.should eq(1)
    t[0].sign.should eq("+")
    t[0].kind.should eq("D")
    t[0].account.should eq("304")

    t = FsFormula.parse("-241К")
    t[0].sign.should eq("-")
    t[0].kind.should eq("C")
    t[0].account.should eq("241")
  end

  it "allows a missing sign on the first token" do
    t = FsFormula.parse("304Д")
    t[0].sign.should eq("+")
    t[0].kind.should eq("D")
  end

  it "reads ОД/ОК/НД/НК before Д/К" do
    FsFormula.parse("+601ОД")[0].kind.should eq("TD")
    FsFormula.parse("+702ОК")[0].kind.should eq("TC")
    FsFormula.parse("+101НК")[0].kind.should eq("OC")
    FsFormula.parse("+123НД")[0].kind.should eq("OD")
  end

  it "parses correspondence and prefix star" do
    t = FsFormula.parse("+304Д/401К+50*Д/411К")
    t.size.should eq(2)
    t[0].kind.should eq("CORR")
    t[0].account.should eq("304")
    t[0].account2.should eq("401")
    t[1].account.should eq("50*")
    t[1].account2.should eq("411")
  end

  it "parses @ references" do
    t = FsFormula.parse("+@bal.c.i.t")
    t[0].kind.should eq("REF")
    t[0].line_code.should eq("bal.c.i.t")
  end

  it "rejects a reversed correspondence" do
    err = FsFormula.validate("+304К/401Д")
    err.should eq("кореспонденцията е Дт/Кт (напр. +304Д/401К)")
  end

  it "rejects unicode minus" do
    err = FsFormula.validate("304Д−241К")
    err.should eq("очакван знак + или -")
  end

  it "rejects unknown @ codes when a statement is given" do
    err = FsFormula.validate("+@bal.zzz", "balance")
    err.should eq("непозната статия @bal.zzz")
  end

  it "accepts known @ codes against the catalog" do
    FsFormula.validate("+@bal.c.i.t", "balance").should be_nil
  end

  it "rejects a formula that is too long" do
    err = FsFormula.validate("+304Д" * 65)
    err.should eq("формулата е твърде дълга")
  end
end
