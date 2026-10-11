module BulgarianNumberToWords
  extend self

  UNITS     = ["", "един", "два", "три", "четири", "пет", "шест", "седем", "осем", "девет"]
  UNITS_FEM = ["", "една", "две", "три", "четири", "пет", "шест", "седем", "осем", "девет"]
  TEENS     = ["десет", "единадесет", "дванадесет", "тринадесет", "четиринадесет", "петнадесет", "шестнадесет", "седемнадесет", "осемнадесет", "деветнадесет"]
  TENS      = ["", "", "двадесет", "тридесет", "четиридесет", "петдесет", "шестдесет", "седемдесет", "осемдесет", "деветдесет"]
  HUNDREDS  = ["", "сто", "двеста", "триста", "четиристотин", "петстотин", "шестстотин", "седемстотин", "осемстотин", "деветстотин"]

  def convert(amount : Float64, currency_code : String = "BGN") : String
    negative = amount < 0
    rounded = Money.round(amount.abs)
    leva = rounded.trunc.to_i64
    stotinki = ((rounded - leva.to_f) * 100.0).round.to_i

    words = String.build do |sb|
      if leva == 0
        sb << "нула"
      else
        sb << convert_long(leva)
      end
      sb << " "
      if currency_code.upcase == "EUR"
        sb << "евро"
      else
        sb << (leva == 1 ? "лев" : "лева")
      end
      if stotinki > 0
        sb << " и "
        sb << convert_small(stotinki, true)
        sb << " "
        if currency_code.upcase == "EUR"
          sb << (stotinki == 1 ? "цент" : "цента")
        else
          sb << (stotinki == 1 ? "стотинка" : "стотинки")
        end
      end
    end
    words = words.strip
    return "" if words.empty?
    capitalized = words[0].upcase + words[1..]
    negative ? "Минус #{words}" : capitalized
  end

  private def convert_long(n : Int64) : String
    return convert_small(n.to_i, false) if n < 1000
    parts = [] of String
    if n >= 1_000_000
      millions = (n // 1_000_000).to_i
      parts << (millions == 1 ? "един милион" : "#{convert_small(millions, false)} милиона")
      n %= 1_000_000
    end
    if n >= 1000
      thousands = (n // 1000).to_i
      parts << (thousands == 1 ? "хиляда" : "#{convert_small(thousands, true)} хиляди")
      n %= 1000
    end
    if n > 0
      rest = convert_small(n.to_i, false)
      parts << (parts.empty? ? rest : "и #{rest}")
    end
    parts.join(" ")
  end

  private def convert_small(n : Int32, feminine : Bool) : String
    h = n // 100
    rem = n % 100
    sb = String.build do |s|
      s << HUNDREDS[h] << " " if h > 0
      if rem > 0
        s << "и " if h > 0 && (rem < 20 || rem % 10 == 0)
        if rem < 10
          s << (feminine ? UNITS_FEM[rem] : UNITS[rem])
        elsif rem < 20
          s << TEENS[rem - 10]
        else
          s << TENS[rem // 10]
          u = rem % 10
          if u > 0
            s << " и " << (feminine ? UNITS_FEM[u] : UNITS[u])
          end
        end
      end
    end
    sb.strip
  end
end
