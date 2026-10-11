# GET /api/admin/audit_logs - List audit log entries (super_admin only)
class Api::Admin::AuditLogs < ApiAction
  include Api::Auth::RequireAuthToken

  get "/api/admin/audit_logs" do
    user = current_user?.not_nil!
    unless user.is_super_admin
      response.status_code = 403
      return json({success: false, message: "Само супер администратори имат достъп"})
    end

    page = (params.get?(:page) || "1").to_i
    per_page = (params.get?(:per_page) || "50").to_i
    per_page = 100 if per_page > 100
    event_filter = params.get?(:event)
    email_filter = params.get?(:email)
    ip_filter = params.get?(:ip)

    query = AuditLogQuery.new

    if event_filter && !event_filter.empty?
      query = query.event(event_filter)
    end

    if ip_filter && !ip_filter.empty?
      query = query.ip_address(ip_filter)
    end

    total = query.clone.select_count

    if email_filter && !email_filter.empty?
      query = query.email.ilike("%#{email_filter}%")
    end

    total = query.clone.select_count

    logs = query
      .created_at.desc_order
      .limit(per_page)
      .offset((page - 1) * per_page)
      .to_a

    total_pages = (total.to_f / per_page).ceil.to_i

    # Stats summary
    stats = audit_stats

    json({
      success: true,
      data:    logs.map { |l|
        {
          id:         l.id,
          event:      l.event,
          email:      l.email,
          ip_address: l.ip_address,
          user_agent: l.user_agent,
          details:    l.details,
          created_at: l.created_at,
        }
      },
      pagination: {
        page:        page,
        per_page:    per_page,
        total:       total,
        total_pages: total_pages,
      },
      stats: stats,
    })
  end

  private def audit_stats
    total = AuditLogQuery.new.select_count
    return nil if total == 0

    # Count by event type using raw SQL for efficiency
    results = AppDatabase.query_all(
      "SELECT event, COUNT(*) as cnt FROM audit_logs GROUP BY event ORDER BY cnt DESC",
      as: {String, Int64}
    )

    ip_results = AppDatabase.query_all(
      "SELECT ip_address, COUNT(*) as cnt, COUNT(DISTINCT email) as emails
       FROM audit_logs GROUP BY ip_address ORDER BY cnt DESC LIMIT 20",
      as: {String, Int64, Int64}
    )

    {
      total:     total,
      by_event:  results.map { |r| {event: r[0], count: r[1]} },
      top_ips:   ip_results.map { |r| {ip: r[0], count: r[1], unique_emails: r[2]} },
    }
  end
end
