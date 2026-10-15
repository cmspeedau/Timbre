Discovery Roadmap
1. Purpose and scope

Help users understand and explore their music collection through accurate metadata, rich relationships, powerful search, radio, and explainable recommendations.

Discovery builds on the canonical library and must never be required for basic playback.

2. Architecture
providers/ — MusicBrainz and other permitted metadata providers.
matching/ — candidate scoring and entity resolution.
relationships/ — artist, contributor, recording, release, and work links.
search/ — indexed catalog queries and facets.
recommendations/ — ranking and diversity rules.
radio/ — queue generation.
provenance/ — source attribution and manual overrides.

Stack: reqwest and serde for provider access, tantivy for the search index.

Use a provider abstraction so external data sources can be replaced. Confirm attribution, licensing, and rate limits for every provider.

3. Implementation phases
Phase 0 — Metadata foundations
- [ ] Define normalized entities and relationship types.
- [ ] Add provider identifiers and provenance fields.
- [ ] Establish confidence scoring for candidate matches.
- [ ] Define manual correction and override semantics.
- [ ] Exit gate: Normalized entities, relationship types, provenance, and confidence scoring are defined.
Phase 1 — Metadata enrichment
- [ ] Integrate MusicBrainz lookups.
- [ ] Match releases using identifiers and track listings.
- [ ] Enrich artist, label, release date, and credit information where available.
- [ ] Cache responses and implement rate limiting.
- [ ] Handle ambiguous and conflicting matches.
- [ ] Preserve user overrides across refreshes.
- [ ] Exit gate: MusicBrainz matching works with caching, rate limiting, and preserved user overrides.
Phase 2 — Search and relationships
- [ ] Search artists, albums, tracks, composers, credits, and labels.
- [ ] Add filtering and faceted results.
- [ ] Link related artists and collaborators.
- [ ] Link recordings to alternate releases when evidence supports the match.
- [ ] Add navigation between works, performances, and recordings where data permits.
- [ ] Evaluate a dedicated search index only when SQLite full-text search is insufficient.
- [ ] Exit gate: Multi-field search, facets, and verified relationship navigation work over the catalog.
Phase 3 — Radio and recommendations
- [ ] Implement radio seeded by a track, artist, album, or genre.
- [ ] Rank candidates by shared credits and musical relationships.
- [ ] Incorporate favorites and listening history.
- [ ] Add repetition penalties and diversity constraints.
- [ ] Explain why recommended items were selected.
- [ ] Allow users to influence or disable recommendation signals.
- [ ] Exit gate: Radio and recommendations produce varied, relevant, explainable results with diversity limits.
Phase 4 — Quality improvements
- [ ] Measure match accuracy against curated fixtures.
- [ ] Add duplicate-entity resolution.
- [ ] Improve search relevance using explicit ranking tests.
- [ ] Support provider outages and stale cached results.
- [ ] Add opt-in feedback for recommendation tuning.
- [ ] Exit gate: Match accuracy and search relevance have regression fixtures and degrade gracefully on provider outages.
4. Task checklist
- [ ] Provider failures do not affect local playback.
- [ ] Metadata matches retain source provenance.
- [ ] Ambiguous matches can be corrected.
- [ ] User overrides are preserved.
- [ ] Search ranking has regression tests.
- [ ] Recommendations avoid excessive repetition.
- [ ] Listening history has configurable privacy controls.
5. API and contracts

Expose interfaces for metadata lookup, match review, relationship traversal, search, radio generation, and recommendations.

Suggested operations:

FindMetadataCandidates(entityID)
ApplyMetadataMatch(entityID, candidateID)
GetRelatedEntities(entityID, relationTypes)
Search(query, filters, pagination)
GenerateRadio(seed, constraints)
GetRecommendations(context, limit)

Use canonical library IDs and return provider provenance when relevant.

6. Testing strategy
Test candidate matching against manually verified releases.
Test ambiguous titles, compilations, and alternate editions.
Test provider throttling, outages, and stale caches.
Test search relevance against curated queries.
Test radio repetition and diversity.
Test that private listening history is not exposed through unauthorized APIs.
7. Acceptance criteria
Metadata matching is explainable and correctable.
Search supports useful multi-field queries and filters.
Related-entity navigation uses verified relationships.
Radio produces varied, relevant selections.
External provider failures degrade gracefully.
User overrides survive metadata refreshes.
8. Dependencies and blockers
Depends on library/ canonical entities and IDs.
Uses server/ for jobs, APIs, and persistence coordination.
Supplies results to client/ and mobile/.
Requires listening-history events from the playback system.
Depends on testing/ for matching and ranking fixtures.
9. Definition of done

Every enrichment or recommendation feature must include data provenance, defined failure behavior, test fixtures, privacy implications, and measurable quality criteria.
