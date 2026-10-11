# Detects how an invoice was paid from AI output + raw document text.
# Card wording varies a lot (карта, POS, ПОС терминал, Visa…); cash is usually "в брой".
module PaymentMethodDetector
  METHODS = ["cash", "card", "bank", "credit", "unknown"]

  struct Result
    getter method : String
    getter raw : String?

    def initialize(@method : String, @raw : String? = nil)
    end
  end

  def self.detect(ai_value : String? = nil, source_text : String? = nil) : Result
    text = normalize_text(source_text)
    from_text = detect_from_text(text)
    from_ai = normalize_ai(ai_value)

    # Card keywords beat a generic "cash" guess — POS slips often mention both каса/карта poorly
    method = if from_text == "card"
               "card"
             elsif from_ai && from_ai != "unknown"
               from_ai
             elsif from_text != "unknown"
               from_text
             else
               "unknown"
             end

    raw = snippet(source_text)
    Result.new(method, raw)
  end

  def self.normalize_ai(value : String?) : String?
    return nil if value.nil?
    v = normalize_text(value)
    return nil if v.empty?

    return "cash" if v.includes?("cash") || v.includes?("брой") || v == "каса"
    return "card" if v.includes?("card") || v.includes?("карт") || v.includes?("pos") || v.includes?("пос") || v.includes?("терминал")
    return "bank" if v.includes?("bank") || v.includes?("превод") || v.includes?("iban") || v.includes?("сметк")
    return "credit" if v.includes?("credit") || v.includes?("отлож") || v.includes?("срок")
    return "unknown" if v.includes?("unknown") || v.includes?("неизв") || v.includes?("n/a") || v.includes?("null")
    nil
  end

  def self.detect_from_text(text : String) : String
    return "unknown" if text.empty?

    return "card" if card?(text)
    return "cash" if cash?(text)
    return "bank" if bank?(text)
    return "credit" if credit?(text)
    "unknown"
  end

  private def self.card?(text : String) : Bool
    text.includes?("карт") ||
      text.includes?("pos") ||
      text.includes?("пос") ||
      text.includes?("терминал") ||
      text.includes?("visa") ||
      text.includes?("mastercard") ||
      text.includes?("maestro") ||
      text.includes?("банкова карта") ||
      text.includes?("debit card") ||
      text.includes?("credit card")
  end

  private def self.cash?(text : String) : Bool
    text.includes?("в брой") ||
      text.includes?("вброй") ||
      text.includes?("налично") ||
      text.includes?("платено в брой") ||
      text.includes?("плащане в брой") ||
      text.includes?("на каса") ||
      text.includes?("в каса") ||
      text.includes?("касата") ||
      text.includes?("cash")
  end

  private def self.bank?(text : String) : Bool
    text.includes?("банков превод") ||
      text.includes?("по сметка") ||
      text.includes?("по банков") ||
      text.includes?("iban") ||
      text.includes?("wire") ||
      text.includes?("безкасов")
  end

  private def self.credit?(text : String) : Bool
    text.includes?("отложено") ||
      text.includes?("срок за плащане") ||
      text.includes?("падеж")
  end

  private def self.normalize_text(value : String?) : String
    return "" if value.nil?
    value.downcase.gsub('\u00a0', ' ').gsub(/\s+/, " ").strip
  end

  private def self.snippet(value : String?) : String?
    return nil if value.nil?
    trimmed = value.strip
    return nil if trimmed.empty?
    trimmed.size > 200 ? trimmed[0, 200] : trimmed
  end
end
