# Folds Bulgarian Cyrillic and Latin bank text into a comparable ASCII key.
# Used so "ПОС" matches "POS" and "НЕВЕЛ 25 ЕООД" matches "Nevel 25 EOOD".
module BankTextNormalizer
  extend self

  CYR_TO_LAT = {
    'а' => "a", 'б' => "b", 'в' => "v", 'г' => "g", 'д' => "d",
    'е' => "e", 'ж' => "zh", 'з' => "z", 'и' => "i", 'й' => "y",
    'к' => "k", 'л' => "l", 'м' => "m", 'н' => "n", 'о' => "o",
    'п' => "p", 'р' => "r", 'с' => "s", 'т' => "t", 'у' => "u",
    'ф' => "f", 'х' => "h", 'ц' => "ts", 'ч' => "ch", 'ш' => "sh",
    'щ' => "sht", 'ъ' => "a", 'ь' => "", 'ю' => "yu", 'я' => "ya",
    'э' => "e", 'ы' => "y", 'ё' => "e", 'і' => "i", 'ї' => "i",
  }

  LEGAL_FORMS = [
    "eood", "ood", "ad", "et", "kd", "kda",
    "ltd", "llc", "gmbh", "srl", "inc", "plc", "sa", "ag",
    "eood", "ood",
  ]

  def fold(text : String) : String
    builder = String::Builder.new
    text.downcase.each_char do |ch|
      if mapped = CYR_TO_LAT[ch]?
        builder << mapped
      elsif ch.ascii_letter? || ch.ascii_number?
        builder << ch
      else
        builder << ' '
      end
    end
    builder.to_s.gsub(/\s+/, " ").strip
  end

  def company_key(text : String) : String
    s = fold(text)
    LEGAL_FORMS.each do |form|
      s = s.gsub(/\b#{Regex.escape(form)}\b/, "")
    end
    s.gsub(/\s+/, " ").strip
  end

  def contains_keyword?(haystack : String, keyword : String) : Bool
    needle = fold(keyword)
    return false if needle.empty?
    fold(haystack).includes?(needle)
  end
end
