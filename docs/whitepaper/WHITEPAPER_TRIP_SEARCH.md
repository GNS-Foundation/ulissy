# TrIP Search
## Spatial-Identity Discovery for the Physical World

**WHITEPAPER**
*Technical Specification & Architecture*
*2026 — Version 0.2 (Implementation Complete)*

**GNS Protocol / ULISSY Foundation**

---

## Abstract

TrIP Search is a discovery layer for the GNS Protocol ecosystem that enables privacy-preserving search over spatially-anchored, trust-scored identities and entities. Where traditional search engines index documents on the web, TrIP Search indexes the **spatial-identity graph** — the network of cryptographically verified entities that have proven their physical existence through TrIP (Trajectory-based Recognition of Identity Proof) trajectories.

Every result in TrIP Search is backed by cryptographic proof of physical existence. Businesses cannot appear at locations where they have no trajectory. Trust scores cannot be purchased or gamed through SEO. Reviews and claims are anchored to verified human identities. This fundamentally inverts the economics of search: ranking is determined by **proof of presence and behavioral trust**, not by ad spend or link authority.

TrIP Search does not expose trajectories. It operates on the public and semi-public outputs of the TrIP/GNS stack — @handles, trust scores, facets, organization namespaces, DIX posts, and gSites — all of which are anchored to private trajectories without revealing them. The search engine sees the *proofs*, never the *paths*.

---

## Table of Contents

1. [The Problem with Search Today](#1-the-problem-with-search-today)
2. [Architecture Overview](#2-architecture-overview)
3. [The Spatial-Identity Graph](#3-the-spatial-identity-graph)
4. [Query Model](#4-query-model)
5. [ULissy Search Primitives](#5-ulissy-search-primitives)
6. [Indexing Architecture](#6-indexing-architecture)
7. [Privacy Model](#7-privacy-model)
8. [Trust-Ranked Results](#8-trust-ranked-results)
9. [Use Cases](#9-use-cases)
10. [Relationship to TrIP Protocol](#10-relationship-to-trip-protocol)
11. [Business Model](#11-business-model)
12. [Roadmap](#12-roadmap)
13. [Conclusion](#13-conclusion)

---

## 1. The Problem with Search Today

### 1.1 The Trust Gap

Search engines were designed to index documents. They answer "what information exists about X?" but cannot answer more fundamental questions:

- **"Is this business actually at this location?"** — Google Maps listings are frequently fake or abandoned. An estimated 11 million fake business listings exist on Google Maps.
- **"Is this reviewer a real person?"** — Fake reviews are a $152 billion industry. Neither Google nor Yelp can cryptographically prove a reviewer physically visited a location.
- **"Is this entity trustworthy?"** — PageRank measures link authority, not behavioral trust. A website can rank #1 with zero physical presence.

### 1.2 The Incentive Problem

Traditional search ranking is an auction, not a meritocracy:

| Factor | Google Search | TrIP Search |
|--------|--------------|-------------|
| Ranking basis | Ad spend + SEO + link authority | Cryptographic proof of presence + trust score |
| Fake listings | ~11 million on Google Maps | Impossible — requires sustained trajectory |
| Fake reviews | $152B industry | Reviewers must have verified @handles |
| Gaming | SEO farms, link buying, click fraud | Cannot fake 6 months of human movement |
| Trust signal | Star ratings (easily gamed) | TrIP trust score (cryptographic) |
| Location proof | Self-reported (honor system) | H3 cell trajectory verification |

### 1.3 The Missing Layer

The internet has a document layer (the web), a naming layer (DNS), and a search layer (Google) for the digital world. For the physical world, it has none of these at the protocol level:

| Function | Digital World | Physical World | GNS Stack |
|----------|--------------|----------------|-----------|
| Naming | DNS | ❌ None | **GNS** (@handles) |
| Identity proof | SSL certificates | ❌ None (biometrics fail) | **TrIP** (trajectory proof) |
| Discovery | Google Search | ❌ None (Google Maps ≠ protocol) | **TrIP Search** |
| Payments | Stripe, PayPal | ❌ Fragmented | **IDUP** (@handle payments) |

TrIP Search fills the discovery gap — a search engine where every result is backed by cryptographic proof of physical existence.

---

## 2. Architecture Overview

### 2.1 Layer Position

TrIP Search is an **application layer** built on the TrIP protocol and GNS identity system. It is not itself a protocol — it consumes protocol data:

```
┌─────────────────────────────────────────────┐
│              TrIP Search                     │  ← Application layer
│         (Discovery & Queries)                │     (this whitepaper)
├─────────────────────────────────────────────┤
│              GNS Protocol                    │  ← Identity layer
│    (@handles, facets, IDUP, DIX, gSites)     │     (GNS whitepaper)
├─────────────────────────────────────────────┤
│             TrIP Protocol                    │  ← Proof layer
│   (breadcrumbs, epochs, TIT, trust scores)   │     (IETF I-D)
├─────────────────────────────────────────────┤
│             ULissy Language                  │  ← Implementation layer
│   (identity, location, time, crypto prims)   │     (ULissy whitepaper)
└─────────────────────────────────────────────┘
```

### 2.2 What Gets Indexed

TrIP Search does **not** index raw trajectories. It indexes the public and semi-public outputs of the GNS ecosystem:

| Data Source | Privacy Level | Example |
|-------------|--------------|---------|
| @handles | Public | `@camiloayerbe` — identity, trust score, verification level |
| Organization namespaces | Public | `acme@corp` — verified business namespace |
| Facets | Declared public | `cafe@luigi` — service facet at H3 cell |
| DIX posts | Public | Geotagged posts in the Decentralized Information Exchange |
| gSites | Public | Personal/business identity pages |
| Epoch summaries | Public (metadata only) | Epoch count, time range, cell diversity — never individual breadcrumbs |
| Trust scores | Public | Computed T = D(t) × S × k (from TrIP spec) |

### 2.3 What Is Never Indexed

| Data | Status | Reason |
|------|--------|--------|
| Raw GPS coordinates | NEVER | Destroyed at quantization (TrIP §3.3) |
| Individual breadcrumbs | NEVER | Private to identity holder |
| Trajectory paths | NEVER | Reconstructable from breadcrumbs |
| Private messages | NEVER | E2E encrypted, only in Envelope |
| Private facets | NEVER | Only public-declared facets are searchable |
| Context digests | NEVER | Sensor fingerprints are one-way hashes |

---

## 3. The Spatial-Identity Graph

### 3.1 Graph Structure

TrIP Search operates on a graph where nodes are verified entities and edges represent spatial, temporal, and identity relationships:

**Node Types:**

| Node | Properties | Source |
|------|-----------|--------|
| Identity | @handle, public key, TIT, trust score, verification level | GNS registry |
| Organization | namespace, member count, domain, verification status | GNS org system |
| Location Cell | H3 index, resolution, activity density, temporal pattern | Aggregated from public facets |
| Facet | protocol prefix, owner identity, associated H3 cells | GNS facet system |
| Content | DIX post, gSite, broadcast | GNS content layer |

**Edge Types:**

| Edge | Meaning | Example |
|------|---------|---------|
| `PRESENT_AT` | Identity has proven presence at H3 cell | `@mario PRESENT_AT 8a194da5329ffff` |
| `MEMBER_OF` | Identity belongs to organization | `@alice MEMBER_OF acme@corp` |
| `PUBLISHED` | Identity published content at location | `@bob PUBLISHED dix_post_7f3a AT 8a194...` |
| `OFFERS` | Facet declares service at location | `delivery@mario OFFERS delivery AT 8a194...` |
| `ADJACENT` | H3 cells share a border (k-ring) | `cell_A ADJACENT cell_B` |
| `TRUSTS` | Cross-endorsement between identities | `@alice TRUSTS @bob` (future: web of trust) |

### 3.2 Spatial Granularity

The graph uses H3 hexagonal cells at multiple resolutions for different query scopes:

| Resolution | Cell Area | Use Case |
|-----------|-----------|----------|
| 7 | ~5.16 km² | City-district discovery |
| 8 | ~0.74 km² | Neighborhood search |
| 9 | ~0.105 km² | Block-level results |
| 10 | ~0.015 km² | Venue-level precision (TrIP default) |
| 11 | ~0.002 km² | Building-level (organizations) |
| 12 | ~0.0003 km² | Room-level (IoT) |

Queries at coarser resolutions aggregate data from finer cells. A resolution-8 search for "verified businesses" aggregates all resolution-10 facets within the ~0.74 km² area.

---

## 4. Query Model

### 4.1 Query Dimensions

Every TrIP Search query operates across four dimensions that map directly to ULissy's language primitives:

| Dimension | Primitive | Query Example |
|-----------|----------|---------------|
| **Space** | `Location`, `H3Cell` | "within 500 meters of here" |
| **Identity** | `Identity`, `Handle` | "trust score above 3.0" |
| **Time** | `Moment`, `Duration` | "active in the last 7 days" |
| **Trust** | `TrustScore` | "verification level ≥ established" |

### 4.2 Query Types

**Spatial Discovery** — "What verified entities exist near this location?"

```
SEARCH entities
  WITHIN 500m OF here
  WHERE trust >= 3.0
  AND has_facet("commerce")
  ORDER BY trust DESC
  LIMIT 20
```

**Identity Lookup** — "What do we know about this entity?"

```
SEARCH identity @camiloayerbe
  INCLUDE trust_score, verification_level, 
          public_facets, epoch_summary, gsite
```

**Temporal-Spatial** — "What was happening in this area recently?"

```
SEARCH activity
  WITHIN cell(8a194da5329ffff).k_ring(2)
  AFTER now - 7.days
  WHERE type IN [dix_post, facet_update, broadcast]
  ORDER BY timestamp DESC
```

**Organization Discovery** — "Which verified organizations operate in this region?"

```
SEARCH organizations
  WITHIN region("Roma, IT")
  WHERE member_count >= 5
  AND namespace_verified = true
  ORDER BY aggregate_trust DESC
```

**Facet Search** — "Who offers this service near me?"

```
SEARCH facets
  WHERE protocol = "delivery"
  WITHIN 2.kilometers OF here
  WHERE owner.trust >= 2.0
  ORDER BY distance ASC
```

**Semantic-Spatial** — "Natural language with spatial context"

```
SEARCH "coffee shop with high trust"
  WITHIN 1.kilometer OF here
```

This combines text matching against facet descriptions, gSite content, and DIX posts with spatial filtering and trust-ranked ordering.

### 4.3 Result Structure

Every search result carries cryptographic provenance:

```json
{
  "results": [
    {
      "entity": {
        "handle": "@luigi",
        "tit": "a7f3b2c1d4e5f6...",
        "trust_score": 4.2,
        "verification_level": "trusted"
      },
      "spatial": {
        "h3_cell": "8a194da5329ffff",
        "resolution": 10,
        "distance_meters": 127,
        "presence_epochs": 42,
        "last_active": "2026-01-28T14:30:00Z"
      },
      "facets": [
        {
          "protocol": "commerce",
          "label": "cafe@luigi",
          "metadata": { "category": "coffee", "hours": "7-19" }
        }
      ],
      "proof": {
        "epoch_count": 42,
        "first_seen": "2025-08-15",
        "spatial_diversity": 0.87,
        "trajectory_continuity": true
      }
    }
  ],
  "query_meta": {
    "total_results": 1,
    "search_radius_m": 500,
    "center_cell": "8a194da532dffff",
    "trust_floor": 3.0,
    "executed_at": "2026-02-02T08:00:00Z"
  }
}
```

The `proof` field is what makes TrIP Search fundamentally different: every result carries verifiable evidence of physical existence, not self-reported claims.

---

## 5. ULissy Search Primitives

### 5.1 Design Principle

Search is not a library import in ULissy — it is a **language construct**. Just as `identity`, `here`, and `now` are first-class primitives, spatial-identity queries are part of the syntax:

```ulissy
// Search is a keyword, not a function call
search nearby(500.meters) where trust > 3.0

// Results are typed and iterable
let results: [SearchResult] = search nearby(1.kilometer)
    where has_facet("commerce")
    and verified == true

for result in results {
    print("\(result.handle) — \(result.distance) away, trust: \(result.trust)")
}
```

### 5.2 New Keywords

| Keyword | Type | Purpose |
|---------|------|---------|
| `search` | Expression | Initiates a spatial-identity query |
| `nearby` | Spatial filter | Distance-based search from current location |
| `within` | Spatial filter | Search within an H3 cell or region |
| `ranked` | Ordering | Trust-weighted result ordering |

### 5.3 Search Expression Grammar

```ebnf
search_expr     = "search" search_target search_filter* search_order? ;
search_target   = nearby_target | within_target | identity_target | text_target ;
nearby_target   = "nearby" ( "(" distance_expr ")" )? ;
within_target   = "within" "(" expr "," distance_expr ")" ;
identity_target = "@" IDENTIFIER ;
text_target     = STRING_LITERAL ;
search_filter   = "where" filter_clause | "," filter_clause ;
filter_clause   = trust_filter | facet_filter | active_filter | org_filter | field_filter ;
trust_filter    = "trust" comparison_op number_literal ;
facet_filter    = "facet" "==" STRING_LITERAL ;
active_filter   = "active" "within" duration_expr ;
org_filter      = "org" "==" STRING_LITERAL ;
field_filter    = IDENTIFIER comparison_op expression ;
search_order    = "ranked" "by" rank_key ("asc" | "desc")? ;
rank_key        = "trust" | "distance" | "recency" | "relevance" ;
```

**Return types by target:**

| Target | Return Type | Example |
|--------|------------|---------|
| `nearby` | `SearchResultSet` | `search nearby(500.meters)` |
| `within` | `SearchResultSet` | `search within(downtown, 5.kilometers)` |
| `@handle` | `Optional<SearchResult>` | `search @alice` |
| `"text"` | `SearchResultSet` | `search "electrician"` |

### 5.4 Type System Extensions

```ulissy
// New built-in types for search
type SearchResult {
    entity:     Identity
    distance:   Distance
    trust:      Float
    facets:     [Facet]
    proof:      PresenceProof
    relevance:  Float
}

type PresenceProof {
    epoch_count:    Int
    first_seen:     Moment
    last_active:    Moment
    spatial_diversity: Float
    continuity:     Bool
}

type SearchQuery {
    center:     H3Cell?
    radius:     Distance?
    trust_floor: Float
    facet_filter: String?
    time_range:  (Moment, Moment)?
    limit:      Int
}

// SearchResult conforms to spatial protocols
impl Locatable for SearchResult {
    computed location: H3Cell = entity.primary_cell
}

impl Trustable for SearchResult {
    computed trustScore: Float = entity.trust
}
```

### 5.5 Reactive Search

ULissy's temporal constructs enable continuous spatial awareness:

```ulissy
// Subscribe to nearby identity changes
every 5.minutes when connected {
    let nearby = search nearby(200.meters) 
        where trust > 2.0
        and has_facet("emergency")
    
    if nearby.count > previous_count {
        notify("New verified responder nearby")
    }
}

// Geofence-triggered discovery
when entering(cell: myWorkCell) {
    let colleagues = search within(myWorkCell.k_ring(1))
        where member_of("acme@corp")
    
    for c in colleagues {
        send("I'm at the office", to: c.handle)
    }
}
```

### 5.6 Privacy-Safe Search Compilation

The ULissy compiler enforces privacy rules at compile time via `reject_private_in_search()`, which blocks five type families from appearing in search contexts:

```ulissy
let results = search nearby(500.meters)

// These compile:
print(results[0].handle)              // OK: public identity
print(results[0].trust)               // OK: public trust score
print(results[0].proof.epoch_count)   // OK: aggregate metadata
print(results[0].proof.spatial_diversity)  // OK: aggregate metric

// These DO NOT compile:
print(results[0].trajectory)      // COMPILE ERROR: Trajectory is private data
print(results[0].breadcrumbs)     // COMPILE ERROR: Breadcrumb contains raw GPS
print(results[0].raw_location)    // COMPILE ERROR: Coordinates not accessible
```

The privacy gate rejects `Breadcrumb`, `Trajectory`, `PrivateKey`, `Identity` (the raw type, not the handle), and `Array<private>` with contextual error messages explaining the violation. This enforcement is verified in the compiler — if the code compiles, it respects the TrIP privacy model.

**`if let` for identity search:** Since `search @handle` returns `Optional<SearchResult>`, ULissy's `if let` binding provides safe unwrapping:

```ulissy
let alice = search @alice

if let peer = alice {
    // peer is SearchResult (unwrapped from Optional)
    print("Found: \(peer.handle) with trust \(peer.trust)")
    let proof = peer.proof
    print("Verified across \(proof.spatial_diversity) diversity")
} else {
    print("Identity not found nearby")
}
```

---

## 6. Indexing Architecture

### 6.1 Design Constraints

The indexing system must satisfy three competing requirements:

1. **Speed** — Spatial queries must resolve in <100ms for real-time mobile use
2. **Privacy** — Individual trajectories must never be reconstructable from index data
3. **Decentralizability** — The index must be distributable across independent nodes

### 6.2 H3 Spatial Index

The primary index structure is a hierarchical H3 cell map:

```
Resolution 7 (district)
  └── Resolution 8 (neighborhood)
       └── Resolution 9 (block)
            └── Resolution 10 (venue) ← TrIP default
                 └── Resolution 11 (building)
                      └── Resolution 12 (room)
```

Each H3 cell at the indexing resolution maintains:

```
CellIndex {
    cell_id:        H3Index          // 64-bit H3 cell identifier
    resolution:     u8               // H3 resolution level (7-12)
    
    // Aggregated presence data (NEVER individual breadcrumbs)
    active_identities:  u32          // Count of identities with presence
    active_orgs:        u32          // Count of organizations
    active_facets:      Vec<FacetSummary>  // Public facet listings
    
    // Temporal activity
    last_activity:      Timestamp    // Most recent public event
    activity_density:   f64          // Events per hour (rolling 7d)
    
    // Content index
    dix_posts:          Vec<PostRef> // References to geotagged content
    gsites:             Vec<SiteRef> // References to identity pages
    
    // Trust aggregates
    mean_trust:         f64          // Average trust of present identities
    max_trust:          f64          // Highest trust score in cell
}
```

### 6.3 Index Partitioning

The spatial index is naturally partitioned by H3 cells, enabling distribution:

**Regional Nodes:** Each node is responsible for a set of resolution-7 cells (typically corresponding to a metropolitan area). A node indexes all finer-resolution data within its region.

**Global Directory:** A lightweight directory maps resolution-7 cells to responsible nodes. This is analogous to DNS root servers — a small, cacheable routing layer.

```
Query flow:
1. Client submits query with center H3 cell
2. Global directory routes to regional node(s)
3. Regional node executes spatial query against local index
4. Results returned with cryptographic proofs
5. Client verifies proof signatures against known public keys
```

### 6.4 Index Updates

The index is fed by public events in the GNS network:

| Event | Index Action | Latency Target |
|-------|-------------|----------------|
| New @handle claimed | Add identity to cell index | < 30 seconds |
| Facet published | Add facet to cell index | < 30 seconds |
| DIX post created | Add content reference | < 10 seconds |
| Trust score updated | Update trust aggregates | < 5 minutes |
| Epoch sealed | Update presence metadata | < 1 minute |
| Identity deactivated | Remove from active sets | < 30 seconds |

Index updates propagate only **public metadata** — the index node never receives breadcrumbs, trajectory data, or private messages.

### 6.5 Inverted Index for Text Search

In addition to the spatial index, a standard inverted index supports text queries across:

- Facet labels and descriptions (`cafe@luigi`, `delivery@mario`)
- gSite content (bio, links, descriptions)
- DIX post text
- Organization names and descriptions
- Handle substrings

Text search results are always intersected with spatial and trust filters. There is no "global text search" without spatial or identity context — this is intentional. TrIP Search is spatial-first by design.

---

## 7. Privacy Model

### 7.1 Core Principle

**The search engine knows WHERE verified entities are, but not HOW they got there.**

This is possible because of the layered architecture:

| Layer | Data | Who Sees It |
|-------|------|------------|
| TrIP breadcrumbs | Individual signed location proofs | **Only the identity holder** |
| TrIP epochs | Merkle roots over breadcrumb ranges | **Only the identity holder** (summary metadata may be public) |
| GNS identity | @handle, public key, trust score | **Public** |
| GNS facets | Protocol-specific identity layers | **Public if declared public** |
| TrIP Search index | Aggregated cell-level presence data | **Search nodes** |

The gap between "identity holder only" and "public" is the privacy boundary. TrIP Search operates entirely on the public side.

### 7.2 What Cannot Be Derived

Even with full access to the search index, an attacker cannot:

1. **Reconstruct a trajectory** — The index contains presence aggregates (identity X has N epochs at cell Y), not ordered breadcrumb sequences. Movement patterns are not recoverable.

2. **Determine exact location** — All spatial data is quantized to H3 cells (minimum ~0.015 km² at resolution 10). Within-cell position is unknown.

3. **Correlate temporal patterns** — The index stores `last_activity` timestamps, not visit histories. An observer cannot determine daily routines.

4. **Link across identities** — TITs (Trajectory Identity Tokens) are derived from the genesis breadcrumb hash. Without the genesis breadcrumb (private), TITs cannot be linked to public keys except through the identity holder's own disclosure.

### 7.3 Consent Model

Searchability is opt-in at every level:

| Level | Default | User Control |
|-------|---------|-------------|
| @handle existence | Discoverable | Can set to private |
| Trust score | Visible | Can hide from search |
| Facets | Private | Must explicitly publish as searchable |
| DIX posts | Public by default | Per-post visibility toggle |
| gSite | Public | Can unpublish |
| Organization membership | Org-controlled | Member can hide affiliation |

An identity with all settings on "private" is invisible to TrIP Search. They still collect breadcrumbs and build trust, but the search index has no data about them.

### 7.4 Query Privacy

Queries themselves carry privacy implications. Two approaches:

**Centralized mode (initial):** Queries go to index nodes. Nodes see the query origin but not the querier's identity (queries are unauthenticated by default). Rate limiting prevents bulk scraping.

**Decentralized mode (future):** Queries are routed through an anonymity layer (onion routing or similar) so index nodes cannot link queries to IP addresses. This requires protocol-level work and is a candidate for future standardization.

---

## 8. Trust-Ranked Results

### 8.1 The TrIP Search Ranking Function

Traditional search uses PageRank (link authority). TrIP Search uses **TrustRank** — a composite score derived entirely from cryptographic evidence:

```
TrustRank(result, query) = 
    α × TripTrust(result.identity) +
    β × SpatialRelevance(result, query.center) +
    γ × TemporalRecency(result.last_active) +
    δ × FacetMatch(result.facets, query.terms)
```

Where:

| Component | Weight | Source | Gameable? |
|-----------|--------|--------|-----------|
| `TripTrust` | α = 0.40 | TrIP trust score T = D(t) × S × k | No — requires sustained physical movement |
| `SpatialRelevance` | β = 0.25 | Haversine distance, normalized | No — H3 cell is cryptographically attested |
| `TemporalRecency` | γ = 0.20 | Gaussian decay from last activity | No — epoch timestamps are signed |
| `FacetMatch` | δ = 0.15 | Text relevance to query terms | Partially — facet text is user-authored |

The weights are configurable per query type. Spatial queries increase β; trust queries increase α.

### 8.2 Why This Cannot Be Gamed

| Attack | Google | TrIP Search |
|--------|--------|-------------|
| Buy ranking position | ✅ Google Ads | ❌ Trust score is cryptographic |
| Create fake listings | ✅ Fake GMB listings | ❌ Requires 100+ breadcrumbs minimum |
| Generate fake reviews | ✅ Review farms | ❌ Reviewers need verified @handles |
| SEO manipulation | ✅ Keyword stuffing, link farms | ❌ 85% of rank is from trajectory proof |
| Sybil attack | ✅ Multiple fake accounts | ❌ Each identity needs physical trajectory |
| Click fraud | ✅ Bot clicks | ❌ No click-based ranking component |

The only component with any gaming potential is `FacetMatch` (15%), which relies on user-authored text. Even this is anchored to a verified identity — spam facets from low-trust identities rank at the bottom.

---

## 9. Use Cases

### 9.1 Verified Local Discovery

**Problem:** A user searches "electrician near me." Google returns SEO-optimized listings, many with fake reviews, some no longer operating at the listed address.

**TrIP Search:** Returns only electricians with:
- A verified @handle (minimum 100 breadcrumbs)
- A `services@handle` facet declaring "electrician" at specific H3 cells
- Proven physical presence at those cells (epoch history)
- Trust score reflecting months or years of trajectory continuity

The user knows the electrician is real, actually operates where they claim, and has a verified identity that persists across the ecosystem.

### 9.2 Anti-Fraud Commerce

**Problem:** Online marketplaces are plagued by fake sellers, dropshipping scams, and non-existent businesses.

**TrIP Search:** A marketplace integrates the TrIP Search API to require:
- All sellers have `commerce@handle` facets with verified H3 presence
- Minimum trust score of 3.0 (300+ breadcrumbs, ~3 months of history)
- Organization namespace verification for business sellers

A buyer searching "handmade ceramics in Trastevere" sees only sellers who have physically been in Trastevere regularly, with cryptographic proof.

### 9.3 Emergency Services Discovery

**Problem:** During a disaster, finding nearby verified medical professionals, shelters, or aid stations is critical but unreliable through traditional search.

**TrIP Search query in ULissy:**

```ulissy
let responders = search nearby(2.kilometers)
    where has_facet("emergency.medical")
    and trust >= 4.0
    and active_within(1.hour)
    ranked by distance asc
```

Results are guaranteed to be real people with verified identities who were physically nearby within the last hour. No fake accounts, no outdated listings.

### 9.4 IoT Device Discovery

**Problem:** Smart city infrastructure (sensors, charging stations, shared vehicles) needs discoverable, verifiable device identities.

**TrIP Search:** IoT devices with TrIP trajectories (even stationary devices have "anchor" breadcrumbs) are discoverable:

```ulissy
let chargers = search within(myCell.k_ring(3))
    where has_facet("iot.ev_charger")
    and entity.type == .device
    and trust >= 1.0
```

Each charger result carries proof that the device has been physically present and operational at its claimed location.

### 9.5 AI Training Data Provenance

**Problem:** AI companies need to prove their training data comes from real humans, not synthetic generation.

**TrIP Search API:**

```
GET /v1/search/content
  ?type=dix_post
  &min_trust=3.0
  &verified_human=true
  &within=global
  &after=2025-01-01
```

Every returned content item is linked to a TrIP-verified human identity. This provides the provenance chain that AI regulation increasingly demands.

### 9.6 Decentralized Social Discovery

**Problem:** Social platforms control discovery algorithms, creating filter bubbles and suppressing content based on corporate policies.

**TrIP Search:** Discovery is spatial and trust-based, not algorithmic:

```ulissy
let feed = search nearby(5.kilometers)
    where type == .dix_post
    and author.trust >= 2.0
    ranked by recency desc
    limit 50
```

The user sees what verified humans near them are saying, ranked by time and trust, not by engagement optimization.

---

## 10. Relationship to TrIP Protocol

### 10.1 Architectural Distinction

It is important to be precise about what TrIP Search is and is not:

| | TrIP Protocol | GNS Protocol | TrIP Search |
|---|---|---|---|
| **Layer** | Proof | Identity | Application |
| **Standardization** | IETF I-D | Proprietary (open API) | Proprietary |
| **Specifies** | Wire format, verification | Naming, facets, payments | Query model, indexing |
| **Interoperability** | Required (MUST/SHOULD) | API contract | Implementation-specific |
| **Data** | Breadcrumbs, epochs, TITs | @handles, facets, records | Aggregated index |

TrIP Search is an **application** that consumes data from TrIP and GNS. It does not define wire formats or verification procedures. It does not require interoperability between independent implementations. It is a product, not a protocol.

### 10.2 Potential Future Protocol Component

One component of TrIP Search could evolve into a protocol: the **discovery mechanism** — how nodes advertise spatial-identity data and how queries are routed in a decentralized network. This is analogous to:

- **DNS-SD (RFC 6763):** Service discovery over DNS
- **mDNS (RFC 6762):** Multicast DNS for local service discovery  
- **IPFS DHT:** Content-addressable routing in a distributed network

If TrIP Search achieves adoption and the decentralized indexing architecture proves viable, a formal **TrIP Discovery Protocol** could be proposed as a separate IETF Internet-Draft. This would standardize:

- How index nodes advertise their regional coverage
- How queries are routed to responsible nodes
- How results carry verifiable cryptographic proofs
- How nodes exchange index updates

This is not planned for the initial version. The initial TrIP Search will operate as a centralized service with a well-defined API, following the same "centralize first, decentralize later" approach that proved successful for the GNS backend.

### 10.3 Dependency on TrIP Specification

TrIP Search depends on the following TrIP protocol constructs (as defined in `draft-ayerbe-trip-protocol-00`):

| TrIP Construct | TrIP Search Usage |
|---------------|-------------------|
| Trust score T = D(t) × S × k | Primary ranking signal |
| H3 cell encoding | Spatial index structure |
| TIT (Trajectory Identity Token) | Pseudonymous result identifiers |
| Epoch metadata | Presence duration evidence |
| Verification levels | Query filter thresholds |
| Ed25519 signatures | Proof verification in results |

Any changes to the TrIP I-D directly affect TrIP Search. The search system will track TrIP revisions and maintain backward compatibility.

---

## 11. Business Model

TrIP Search follows the same COSS (Commercial Open Source Software) model as the broader GNS ecosystem:

### 11.1 Open Components

| Component | License | Purpose |
|-----------|---------|---------|
| Query specification | Public (this whitepaper) | Anyone can build TrIP Search clients |
| ULissy search primitives | MIT (part of ULissy) | Language-level search constructs |
| Client SDK | MIT | JavaScript/Python/Dart query libraries |

### 11.2 Commercial Components

| Product | Model | Target |
|---------|-------|--------|
| **TrIP Search API** | Usage-based (per query) | Developers integrating spatial discovery |
| **Managed Index Nodes** | Subscription | Companies running regional search infrastructure |
| **Enterprise Search** | Contract + SLA | Organizations needing private spatial-identity search |
| **Search Analytics** | Subscription | Businesses tracking their discoverability and trust metrics |

### 11.3 Pricing Structure

| Tier | Monthly | Queries/Month | Features |
|------|---------|---------------|----------|
| Developer | $0 | 10,000 | Basic spatial search, 1 region |
| Startup | $99/mo | 100,000 | All query types, 5 regions |
| Business | $299/mo | 1,000,000 | Analytics, priority indexing, all regions |
| Enterprise | Custom | Unlimited | Private deployment, SLA, custom ranking |

### 11.4 Revenue Synergy

TrIP Search creates a flywheel with existing GNS revenue streams:

```
More @handles → Richer spatial-identity graph → Better search results
     ↑                                                    ↓
     └──── More businesses claim facets ← More user queries
```

Each component reinforces the others:
- Search drives @handle adoption (you need a verified identity to be discoverable)
- Facets drive search quality (businesses declare services to be found)
- Trust scores drive ranking quality (longer trajectories = higher ranking)
- Organization namespaces drive enterprise adoption (companies want to be discovered)

---

## 12. Roadmap

### Phase 0: Language Foundation ✅ (Complete — ULissy v0.4.2)

- ✅ Design ULissy search primitives (`search`, `nearby`, `within`, `ranked`, `by`)
- ✅ Extend grammar specification with search expressions (4 targets, typed filters, semantic ranking)
- ✅ Add `SearchResult`, `PresenceProof`, `TrustScore`, `SearchQuery`, `SpatialFilter`, `IdentityFilter` to type system
- ✅ Implement search expression parsing in compiler (all four query types)
- ✅ Compile-time privacy enforcement (`reject_private_in_search()` gate)
- ✅ `if let` optional binding for identity search results
- ✅ Typed code generation emitting `gns_search::query()` builder pattern
- ✅ Unit suffix aliases (singular, abbreviated: `km`, `m`, `hour`, `min`)
- ✅ End-to-end verified: `ulissy check` and `ulissy build` pass on comprehensive search test file

### Phase 1: Centralized Search API (Current — v0.1)

- ⏳ **`gns-search` Rust crate** — Runtime implementation of the `gns_search::query()` builder that compiled ULissy targets
- ⏳ Build H3 spatial index over existing GNS public data (PostgreSQL + H3 extension)
- ⏳ Implement core query engine (spatial + trust + temporal filtering)
- ⏳ REST API: `GET /v1/search` with spatial, identity, and facet query parameters
- ⏳ Index updater consuming GNS backend events
- ⏳ Basic text search (inverted index over facets, gSites, DIX posts)

### Phase 2: Trust-Ranked Results (v0.2)

- ⏳ Implement TrustRank scoring function
- ⏳ Result proof generation (cryptographic provenance in responses)
- ⏳ Query analytics and relevance tuning
- ⏳ ULissy `search` keyword compiles to REST API calls
- ⏳ Client SDKs (JavaScript, Python, Dart)

### Phase 3: Decentralized Indexing (v1.0)

- ⏳ Regional node architecture with global directory
- ⏳ Index partitioning by H3 resolution-7 cells
- ⏳ Node-to-node index synchronization
- ⏳ Query routing across regional nodes
- ⏳ Evaluate TrIP Discovery Protocol for IETF submission

---

## 13. Conclusion

TrIP Search completes the GNS ecosystem's answer to a question the internet has never properly solved: **how do you discover verified, trustworthy entities in the physical world?**

Google indexes documents. DNS indexes domains. TrIP Search indexes **spatially-anchored, cryptographically-verified identities**. Every result carries proof of physical existence. Every ranking is earned through trajectory, not purchased through advertising.

The architecture rests on a crucial privacy insight: you can build a useful spatial discovery engine without ever seeing a single trajectory. The search layer operates on the *proofs* (trust scores, epoch counts, H3 cell presence) while the *paths* (breadcrumbs, GPS data, movement patterns) remain exclusively with the identity holder. This is only possible because the TrIP protocol was designed with this separation from the beginning.

By embedding search as a first-class primitive in ULissy, spatial-identity discovery becomes as natural as variable declaration — `search nearby(500.meters) where trust > 3.0 ranked by distance asc` is a complete, type-safe, privacy-preserving query in a single line. As of ULissy v0.4.2, this line compiles through the full pipeline — lexer, parser, type checker with privacy enforcement, and code generator — producing idiomatic Rust that targets the `gns_search` query builder.

The era of trusting search results because an algorithm said so is ending. The era of trusting search results because cryptography proved them is beginning.

**HUMANS PREVAIL.**

---

## References

- **TrIP Protocol:** `draft-ayerbe-trip-protocol-00`, IETF Internet-Draft, February 2026. https://datatracker.ietf.org/doc/draft-ayerbe-trip-protocol/
- **ULissy Language Whitepaper:** Version 0.4.2, ULISSY Foundation / GNS Protocol, 2025-2026.
- **GNS Protocol:** Geospatial Naming System specification, ULISSY s.r.l.
- **H3 Spatial Index:** Uber Technologies, https://h3geo.org/
- **Ed25519:** RFC 8032 — Edwards-Curve Digital Signature Algorithm
- **SHA-256:** RFC 6234 — US Secure Hash Algorithms
- **CBOR:** RFC 8949 — Concise Binary Object Representation
- **DNS-SD:** RFC 6763 — DNS-Based Service Discovery
- **mDNS:** RFC 6762 — Multicast DNS
- **Provisional Patent:** #63/948,788 — Proof-of-Trajectory methodology

---

*TrIP Search Whitepaper v0.2 — Implementation Complete*
*GNS Protocol / ULISSY Foundation*
*February 2026*
