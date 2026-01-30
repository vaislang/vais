# Vais DNS Configuration Guide

Complete DNS setup for vais-lang.org and subdomains.

## Overview

The Vais project requires DNS configuration for:
- Main domain: `vais-lang.org`
- WWW subdomain: `www.vais-lang.org`
- Documentation: `docs.vais-lang.org`
- Playground: `play.vais-lang.org`

## Prerequisites

- Domain ownership/registrar access (vais-lang.org)
- DNS provider account (Namecheap, GoDaddy, Route53, Cloudflare, etc.)
- Hosting platform choice (GitHub Pages or Vercel)

## GitHub Pages DNS Configuration

Use this configuration if deploying via GitHub Pages.

### Step 1: Get GitHub Pages IP Addresses

GitHub provides four A record IP addresses:

```
185.199.108.153
185.199.109.153
185.199.110.153
185.199.111.153
```

### Step 2: Configure Primary Domain (vais-lang.org)

#### In Your Registrar's DNS Manager

**Add A Records:**

| Type | Host | Value | TTL |
|------|------|-------|-----|
| A | @ | 185.199.108.153 | 3600 |
| A | @ | 185.199.109.153 | 3600 |
| A | @ | 185.199.110.153 | 3600 |
| A | @ | 185.199.111.153 | 3600 |

**Add AAAA Records (IPv6):**

| Type | Host | Value | TTL |
|------|------|-------|-----|
| AAAA | @ | 2606:50c0:8000::153 | 3600 |
| AAAA | @ | 2606:50c0:8001::153 | 3600 |
| AAAA | @ | 2606:50c0:8002::153 | 3600 |
| AAAA | @ | 2606:50c0:8003::153 | 3600 |

### Step 3: Configure WWW Subdomain

**Add CNAME Record:**

| Type | Host | Value | TTL |
|------|------|-------|-----|
| CNAME | www | sswoo88.github.io | 3600 |

### Step 4: Configure Subdomains (GitHub Pages)

If deploying docs and playground to same GitHub Pages site:

**Documentation Subdomain:**

| Type | Host | Value | TTL |
|------|------|-------|-----|
| CNAME | docs | sswoo88.github.io | 3600 |

**Playground Subdomain:**

| Type | Host | Value | TTL |
|------|------|-------|-----|
| CNAME | play | sswoo88.github.io | 3600 |

**Note:** These would require separate branches/deployments configured in GitHub Pages settings.

### Step 5: Verify DNS Propagation

```bash
# Check main domain
nslookup vais-lang.org
dig vais-lang.org

# Check www subdomain
nslookup www.vais-lang.org
dig CNAME www.vais-lang.org

# Check other subdomains
nslookup docs.vais-lang.org
nslookup play.vais-lang.org
```

Expected output for A records:
```
vais-lang.org has address 185.199.108.153
vais-lang.org has address 185.199.109.153
vais-lang.org has address 185.199.110.153
vais-lang.org has address 185.199.111.153
```

Expected output for CNAME:
```
www.vais-lang.org canonical name = sswoo88.github.io
```

## Vercel DNS Configuration

Use this configuration if deploying via Vercel.

### Step 1: Add Domain to Vercel

1. Go to Vercel dashboard
2. Select project (website, docs, or playground)
3. Settings → Domains
4. Enter domain name
5. Vercel provides DNS instructions

### Step 2: Configure Primary Domain

Vercel typically provides one of these options:

**Option A: Using Nameservers (Recommended)**
```
Change your registrar's nameservers to Vercel's:
ns1.vercel.com
ns2.vercel.com
```

**Option B: Using CNAME**

| Type | Host | Value | TTL |
|------|------|-------|-----|
| CNAME | @ | cname.vercel.com | 3600 |

### Step 3: Configure Subdomains

Vercel deployments are typically:

**Documentation:**
```
docs.vais-lang.org → Separate Vercel project
CNAME: cname.vercel.com
```

**Playground:**
```
play.vais-lang.org → Separate Vercel project
CNAME: cname.vercel.com
```

### Step 4: Verify Domain Ownership

Vercel will ask you to verify ownership. Options:
- Add TXT record (provided by Vercel)
- Or verify via GitHub integration (automatic)

Add TXT record:

| Type | Host | Value | TTL |
|------|------|-------|-----|
| TXT | _vercel | (Vercel-provided value) | 3600 |

## Hybrid Configuration Example

If using GitHub Pages for website and Vercel for docs/playground:

### Primary Domain Routes

```
vais-lang.org (main website)
  → GitHub Pages A records

www.vais-lang.org (also main website)
  → GitHub Pages CNAME (www.sswoo88.github.io)

docs.vais-lang.org (documentation)
  → Vercel CNAME (cname.vercel.com)

play.vais-lang.org (playground)
  → Vercel CNAME (cname.vercel.com)
```

### Complete DNS Record Table

| Type | Host | Value | Service | TTL |
|------|------|-------|---------|-----|
| A | @ | 185.199.108.153 | GitHub | 3600 |
| A | @ | 185.199.109.153 | GitHub | 3600 |
| A | @ | 185.199.110.153 | GitHub | 3600 |
| A | @ | 185.199.111.153 | GitHub | 3600 |
| AAAA | @ | 2606:50c0:8000::153 | GitHub | 3600 |
| AAAA | @ | 2606:50c0:8001::153 | GitHub | 3600 |
| AAAA | @ | 2606:50c0:8002::153 | GitHub | 3600 |
| AAAA | @ | 2606:50c0:8003::153 | GitHub | 3600 |
| CNAME | www | sswoo88.github.io | GitHub | 3600 |
| CNAME | docs | cname.vercel.com | Vercel | 3600 |
| CNAME | play | cname.vercel.com | Vercel | 3600 |

## Common Registrars - Setup Steps

### Namecheap

1. Login to Namecheap
2. Domain List → vais-lang.org → Manage
3. Advanced DNS tab
4. Add A records: Type=A, Host=@, Value=(IP), TTL=3600
5. Add CNAME records: Type=CNAME, Host=(subdomain), Value=(target), TTL=3600

### GoDaddy

1. Login to GoDaddy
2. My Domains → vais-lang.org
3. DNS section
4. Add Records:
   - For A records: Select A, enter host (@) and value (IP)
   - For CNAME: Select CNAME, enter host (subdomain) and value (target)

### Google Domains / Google Cloud DNS

1. Login to Google Domains
2. DNS settings for vais-lang.org
3. Custom records section
4. Add A, AAAA, and CNAME records as specified

### Cloudflare

1. Add domain to Cloudflare account
2. Copy Cloudflare nameservers to registrar
3. Cloudflare DNS section
4. Add records matching your configuration

### AWS Route 53

1. Create hosted zone for vais-lang.org
2. Copy nameservers to registrar (if not transferred)
3. Create records:
   - A records (4)
   - AAAA records (4)
   - CNAME records

## DNS Propagation & Testing

### Understanding TTL (Time To Live)

- Lower TTL (300-3600): Changes propagate faster, more DNS queries
- Higher TTL (86400): Changes take longer, fewer DNS queries
- Recommended: 3600 (1 hour) for updates

### Checking Propagation Status

```bash
# Global DNS propagation checker
nslookup vais-lang.org
nslookup -type=MX vais-lang.org  # Mail records (if needed)

# Detailed DNS information
dig vais-lang.org +short
dig vais-lang.org

# Check specific nameserver
dig @8.8.8.8 vais-lang.org
dig @1.1.1.1 vais-lang.org

# Online tools
# https://dnschecker.org
# https://mxtoolbox.com/mxlookup.aspx
# https://www.whatsmydns.net/
```

### Typical Propagation Timeline

| Time | Status |
|------|--------|
| 0-1 min | Changes saved at registrar |
| 5-15 min | Global nameservers begin updating |
| 15 min - 2 hrs | Most DNS servers updated |
| 2-48 hrs | Full global propagation |

In practice, most users see changes within 5-30 minutes.

## Troubleshooting DNS Issues

### Issue: Domain points to wrong location

**Solution:**
1. Verify DNS records in registrar: `nslookup vais-lang.org`
2. Check for typos in IP addresses or CNAME values
3. Ensure correct record type (A vs CNAME)
4. Wait for TTL to expire and clear browser cache

### Issue: Subdomain not resolving

**Solution:**
```bash
# Check if subdomain record exists
nslookup docs.vais-lang.org
dig docs.vais-lang.org

# If not found:
1. Add CNAME record to registrar
2. Wait 5-15 minutes
3. Test again with nslookup
```

### Issue: HTTPS certificate error

**Solution:**
1. Ensure domain is properly configured in GitHub Pages/Vercel settings
2. Wait 10 minutes for certificate issuance (Let's Encrypt)
3. Check GitHub Pages settings → "Enforce HTTPS" is enabled
4. Verify custom domain field matches the domain you're accessing

### Issue: GitHub Pages shows 404

**Solution:**
1. Ensure GitHub Pages is enabled in repository settings
2. Verify custom domain is set correctly
3. Check DNS A records are added
4. Wait for certificate issuance
5. Ensure `dist/index.html` exists in built site

## DNSSEC Configuration (Optional)

For enhanced security, enable DNSSEC:

### Enable DNSSEC

1. Contact registrar for DNSSEC support
2. Enable DNSSEC in DNS provider settings
3. Add DS records provided by DNS provider to registrar
4. Verify with: `dig +dnssec vais-lang.org`

## Email Configuration (Future)

If needing email for domain (support@vais-lang.org):

**Add MX Records:**

| Type | Host | Value | Priority | TTL |
|------|------|-------|----------|-----|
| MX | @ | mail.provider.com | 10 | 3600 |
| TXT | @ | v=spf1 include:provider.com ~all | - | 3600 |
| CNAME | _dmarc | (provider value) | - | 3600 |

## Monitoring DNS Health

### Regular Checks

```bash
# Weekly DNS verification
nslookup vais-lang.org
nslookup www.vais-lang.org
nslookup docs.vais-lang.org
nslookup play.vais-lang.org

# Check for issues
dig +trace vais-lang.org  # Shows full resolution path
```

### Monitoring Tools

- [DNS Checker](https://dnschecker.org)
- [MX Toolbox](https://mxtoolbox.com)
- [What's My DNS](https://www.whatsmydns.net/)
- [StatusCake DNS](https://statuspage.statuscake.com/)

## DNS Security Best Practices

1. **Use HTTPS:** Enforce HTTPS in GitHub Pages/Vercel settings
2. **Monitor DNS:** Regular checks with nslookup/dig
3. **Update Records:** Keep DNS records clean, remove unused entries
4. **Enable DNSSEC:** If supported by provider
5. **Backup Configuration:** Document all DNS records
6. **Regular Audits:** Check DNS configuration quarterly

## Rollback & Recovery

### If Domain Breaks

```bash
# Emergency: Point to temporary location
# Change A records to temporary server IP
# Or disable CNAME records temporarily

# Recover:
1. Revert DNS changes at registrar
2. Allow 5-15 minutes for propagation
3. Test with nslookup
```

### DNS Record Backup

Keep record of all DNS entries:

```
vais-lang.org:
- A: 185.199.108.153 / 185.199.109.153 / 185.199.110.153 / 185.199.111.153
- AAAA: 2606:50c0:8000::153 / 2606:50c0:8001::153 / 2606:50c0:8002::153 / 2606:50c0:8003::153

www.vais-lang.org:
- CNAME: sswoo88.github.io

docs.vais-lang.org:
- CNAME: cname.vercel.com

play.vais-lang.org:
- CNAME: cname.vercel.com
```

## Next Steps

1. Choose hosting platform (GitHub Pages recommended)
2. Add DNS records to registrar
3. Wait for propagation (5-30 minutes typical)
4. Test with `nslookup` command
5. Verify in GitHub Pages / Vercel settings
6. Test accessing websites in browser

## Additional Resources

- [GitHub Pages DNS Documentation](https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-site/managing-a-custom-domain-for-your-github-pages-site)
- [Vercel Domains Documentation](https://vercel.com/docs/concepts/projects/domains)
- [DNS Propagation Guide](https://www.cloudflare.com/learning/dns/dns-propagation/)
- [DNSSEC Guide](https://www.cloudflare.com/learning/dns/dnssec/what-is-dnssec/)
