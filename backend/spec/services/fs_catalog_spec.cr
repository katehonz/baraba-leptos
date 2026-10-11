require "../spec_helper"

describe FsCatalog do
  it "loads four statements with unique codes and existing parents" do
    FsCatalog::STATEMENTS.each do |st|
      rows = FsCatalog.lines(st)
      rows.size.should be > 0
      codes = rows.map(&.code)
      codes.uniq.size.should eq(codes.size)
      rows.each do |row|
        FsCatalog::KINDS.includes?(row.kind).should be_true
        if parent = row.parent
          next if parent.blank?
          FsCatalog.has_code?(st, parent).should be_true
        end
      end
    end
    FsCatalog.lines("balance").size.should eq(92)
  end

  it "nests в т.ч. under the numbered line, not the group" do
    FsCatalog.find("balance", "bal.b.ii.1a").not_nil!.parent.should eq("bal.b.ii.1")
    FsCatalog.find("balance", "bal.b.ii.1b").not_nil!.parent.should eq("bal.b.ii.1")
    FsCatalog.find("pl", "pl.e.2a").not_nil!.parent.should eq("pl.e.2")
  end

  it "keeps group totals free of grandchild breakdowns" do
    ii_t = FsCatalog.defaults("balance")["bal.b.ii.t"]
    ii_t.includes?("bal.b.ii.1a").should be_false
    ii_t.includes?("bal.b.ii.1b").should be_false
    ii_t.includes?("@bal.b.ii.1+").should be_true

    e_t = FsCatalog.defaults("pl")["pl.e.t"]
    e_t.includes?("pl.e.tax").should be_false
    e_t.includes?("pl.e.profit").should be_false
    e_t.should eq("+@pl.e.op+@pl.e.fin")
  end

  it "parses every default formula against its statement" do
    FsCatalog::STATEMENTS.each do |st|
      FsCatalog.defaults(st).each do |code, formula|
        FsCatalog.has_code?(st, code).should be_true
        err = FsFormula.validate(formula, st)
        err.should eq(nil)
      end
    end
  end

  it "puts new cash-flow and equity stubs in parent totals" do
    FsCatalog.defaults("cashflow")["cf.inv.net"].includes?("@cf.inv.acq").should be_true
    FsCatalog.defaults("equity")["eq.close"].includes?("@eq.loss_cover").should be_true
    FsCatalog.has_code?("equity", "eq.loss_cover").should be_true
  end
end
