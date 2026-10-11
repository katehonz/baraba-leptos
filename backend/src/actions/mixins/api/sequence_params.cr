module Api::Sequences::Params
  private def sequence_body : JSON::Any
    params.from_json
  end

  private def read_name(body : JSON::Any) : String
    body["name"]?.try(&.as_s?).try(&.strip) || ""
  end

  private def read_next_number(body : JSON::Any) : Int64?
    raw = body["next_number"]?
    return nil if raw.nil?
    node = raw.not_nil!
    if n = node.as_i64?
      return n
    end
    if n = node.as_i?
      return n.to_i64
    end
    if s = node.as_s?
      text = s.strip
      return nil if text.blank?
      return text.to_i64?
    end
    nil
  end

  private def read_document_types(body : JSON::Any) : Array(String)
    raw = body["document_types"]?
    arr = raw.try(&.as_a?)
    return [] of String unless arr
    arr.not_nil!.compact_map { |item| item.as_s? }.map(&.strip).reject(&.blank?).uniq
  end
end
