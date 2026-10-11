# Parser for NSS-1 financial-statement formulas (port of bagabuch fs_formula.baga).
# Tokens: +304Д -241К +601ОД +702ОК +101НК +304Д/401К +@bal.c.i.t
# Left-to-right, no brackets. UTF-8 kinds (ОД before Д). ASCII + / - only.

class FsParseError < Exception
end

struct FsToken
  getter sign : String
  getter kind : String
  getter account : String?
  getter account2 : String?
  getter line_code : String?

  def initialize(@sign : String, @kind : String, @account : String? = nil, @account2 : String? = nil, @line_code : String? = nil)
  end

  def negative? : Bool
    sign == "-"
  end
end

module FsFormula
  extend self

  MAX_LEN    = 500
  MAX_TOKENS =  64

  DEBIT_KINDS  = {"D", "TD", "OD"}
  CREDIT_KINDS = {"C", "TC", "OC"}

  def parse(raw : String, statement : String? = nil) : Array(FsToken)
    s = raw.strip
    return [] of FsToken if s.empty?
    raise FsParseError.new("формулата е твърде дълга") if s.size > MAX_LEN

    tokens = [] of FsToken
    i = 0
    n = s.size
    first = true

    while i < n
      i = skip_ws(s, i)
      break if i >= n

      sign = "+"
      ch = s[i]
      if ch == '+'
        sign = "+"
        i += 1
        first = false
      elsif ch == '-'
        sign = "-"
        i += 1
        first = false
      else
        unless first
          raise FsParseError.new("очакван знак + или -")
        end
        first = false
      end

      i = skip_ws(s, i)
      raise FsParseError.new("формулата свършва след знака") if i >= n

      if s[i] == '@'
        i += 1
        code = read_code(s, i)
        raise FsParseError.new("липсва код след @") if code.empty?
        i += code.size
        if st = statement
          unless FsCatalog.has_code?(st, code)
            raise FsParseError.new("непозната статия @#{code}")
          end
        end
        tokens << FsToken.new(sign, "REF", nil, nil, code)
      else
        acc = read_acc(s, i)
        raise FsParseError.new("липсва сметка") if acc.empty?
        i += acc.size
        k, klen = read_kind(s, i)
        raise FsParseError.new("липсва Д/К/ОД/ОК/НД/НК след #{acc}") if k.empty?
        i += klen
        i = skip_ws(s, i)
        if i < n && s[i] == '/'
          i += 1
          i = skip_ws(s, i)
          acc2 = read_acc(s, i)
          raise FsParseError.new("липсва кредитна сметка след /") if acc2.empty?
          i += acc2.size
          k2, k2len = read_kind(s, i)
          raise FsParseError.new("липсва Д или К след втората сметка") if k2.empty?
          i += k2len
          unless DEBIT_KINDS.includes?(k) && CREDIT_KINDS.includes?(k2)
            raise FsParseError.new("кореспонденцията е Дт/Кт (напр. +304Д/401К)")
          end
          tokens << FsToken.new(sign, "CORR", acc, acc2, nil)
        else
          tokens << FsToken.new(sign, k, acc, nil, nil)
        end
      end

      raise FsParseError.new("формулата е твърде дълга") if tokens.size > MAX_TOKENS
    end

    tokens
  end

  def validate(raw : String, statement : String? = nil) : String?
    parse(raw, statement)
    nil
  rescue ex : FsParseError
    ex.message
  end

  private def skip_ws(s : String, i : Int32) : Int32
    n = s.size
    while i < n
      ch = s[i]
      break unless ch == ' ' || ch == '\t'
      i += 1
    end
    i
  end

  private def read_acc(s : String, i : Int32) : String
    start = i
    n = s.size
    while i < n
      ch = s[i]
      break unless ch.ascii_number? || ch == '*'
      i += 1
    end
    s[start, i - start]
  end

  private def read_code(s : String, i : Int32) : String
    start = i
    n = s.size
    while i < n
      ch = s[i]
      break if ch == '+' || ch == '-' || ch == ' ' || ch == '\t'
      i += 1
    end
    s[start, i - start]
  end

  # Returns {internal_kind, char_length}. ОД/ОК/НД/НК before Д/К.
  private def read_kind(s : String, i : Int32) : {String, Int32}
    rest = i < s.size ? s[i..-1] : ""
    return {"TD", 2} if rest.starts_with?("ОД")
    return {"TC", 2} if rest.starts_with?("ОК")
    return {"OD", 2} if rest.starts_with?("НД")
    return {"OC", 2} if rest.starts_with?("НК")
    return {"D", 1} if rest.starts_with?("Д")
    return {"C", 1} if rest.starts_with?("К")
    {"", 0}
  end
end
