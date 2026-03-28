#!/usr/bin/env python3
"""Audit log viewer for Baraba.

Usage (inside backend container):
  python3 /app/scripts/audit_log.py                    # last 50 events
  python3 /app/scripts/audit_log.py -n 100             # last 100 events
  python3 /app/scripts/audit_log.py -e register         # only registrations
  python3 /app/scripts/audit_log.py -e login_failed     # failed logins
  python3 /app/scripts/audit_log.py --email user@x.com  # by email
  python3 /app/scripts/audit_log.py --ip 1.2.3.4        # by IP
  python3 /app/scripts/audit_log.py --stats              # summary stats
  python3 /app/scripts/audit_log.py --ips                # unique IPs with counts
"""

import argparse
import os
import subprocess
import sys

DATABASE_URL = os.environ.get("DATABASE_URL", "")


def run_sql(sql, database_url=None):
    url = database_url or DATABASE_URL
    if not url:
        print("ERROR: DATABASE_URL not set")
        sys.exit(1)
    result = subprocess.run(
        ["psql", url, "-t", "-A", "-F", "\t", "-c", sql],
        capture_output=True, text=True
    )
    if result.returncode != 0:
        print(f"SQL error: {result.stderr.strip()}")
        sys.exit(1)
    rows = []
    for line in result.stdout.strip().split("\n"):
        if line:
            rows.append(line.split("\t"))
    return rows


def print_table(headers, rows):
    if not rows:
        print("(no results)")
        return
    widths = [len(h) for h in headers]
    for row in rows:
        for i, col in enumerate(row):
            if i < len(widths):
                widths[i] = max(widths[i], len(col))
    fmt = "  ".join(f"{{:<{w}}}" for w in widths)
    print(fmt.format(*headers))
    print("  ".join("-" * w for w in widths))
    for row in rows:
        padded = row + [""] * (len(headers) - len(row))
        print(fmt.format(*padded[:len(headers)]))


def cmd_list(args):
    where = []
    if args.event:
        where.append(f"event = '{args.event}'")
    if args.email:
        where.append(f"email ILIKE '%{args.email}%'")
    if args.ip:
        where.append(f"ip_address = '{args.ip}'")

    where_clause = "WHERE " + " AND ".join(where) if where else ""

    sql = f"""
        SELECT
            to_char(created_at AT TIME ZONE 'Europe/Sofia', 'YYYY-MM-DD HH24:MI:SS') as time,
            event,
            email,
            ip_address,
            COALESCE(LEFT(user_agent, 60), '') as ua,
            COALESCE(details, '') as details
        FROM audit_logs
        {where_clause}
        ORDER BY created_at DESC
        LIMIT {args.n}
    """
    rows = run_sql(sql)
    print_table(["Time", "Event", "Email", "IP", "User-Agent", "Details"], rows)


def cmd_stats(args):
    sql = """
        SELECT event, COUNT(*) as cnt,
               COUNT(DISTINCT email) as emails,
               COUNT(DISTINCT ip_address) as ips
        FROM audit_logs
        GROUP BY event
        ORDER BY cnt DESC
    """
    rows = run_sql(sql)
    print_table(["Event", "Count", "Unique Emails", "Unique IPs"], rows)

    print()
    sql2 = """
        SELECT
            to_char(created_at AT TIME ZONE 'Europe/Sofia', 'YYYY-MM-DD') as day,
            COUNT(*) FILTER (WHERE event = 'register') as registrations,
            COUNT(*) FILTER (WHERE event = 'login') as logins,
            COUNT(*) FILTER (WHERE event = 'login_failed') as failed
        FROM audit_logs
        GROUP BY day
        ORDER BY day DESC
        LIMIT 14
    """
    rows2 = run_sql(sql2)
    print_table(["Day", "Registrations", "Logins", "Failed Logins"], rows2)


def cmd_ips(args):
    sql = """
        SELECT ip_address,
               COUNT(*) as total,
               COUNT(*) FILTER (WHERE event = 'register') as regs,
               COUNT(*) FILTER (WHERE event = 'login') as logins,
               COUNT(*) FILTER (WHERE event = 'login_failed') as fails,
               COUNT(DISTINCT email) as emails,
               MAX(to_char(created_at AT TIME ZONE 'Europe/Sofia', 'YYYY-MM-DD HH24:MI')) as last_seen
        FROM audit_logs
        GROUP BY ip_address
        ORDER BY total DESC
        LIMIT {n}
    """.format(n=args.n)
    rows = run_sql(sql)
    print_table(["IP", "Total", "Regs", "Logins", "Fails", "Emails", "Last Seen"], rows)


def main():
    parser = argparse.ArgumentParser(description="Baraba audit log viewer")
    parser.add_argument("-n", type=int, default=50, help="Number of rows (default: 50)")
    parser.add_argument("-e", "--event", help="Filter by event type")
    parser.add_argument("--email", help="Filter by email (partial match)")
    parser.add_argument("--ip", help="Filter by IP address")
    parser.add_argument("--stats", action="store_true", help="Show summary statistics")
    parser.add_argument("--ips", action="store_true", help="Show unique IPs with counts")

    args = parser.parse_args()

    if args.stats:
        cmd_stats(args)
    elif args.ips:
        cmd_ips(args)
    else:
        cmd_list(args)


if __name__ == "__main__":
    main()
