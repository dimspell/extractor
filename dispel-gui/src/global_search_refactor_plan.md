# Global Search Performance Improvement Plan

## Current Issues
- Laggy typing experience (searches on every keystroke)
- Full result list re-renders on each update
- String allocations in hot path
- No virtualization for large result sets

## Phase 0: Directory Restructuring (Foundation)
- [ ] Create `dispel-gui/src/components/global_search/` directory
- [ ] Move `global_search.rs` → `components/global_search/mod.rs`
- [ ] Move search-related view code → `components/global_search/view.rs`
- [ ] Update all imports to use new component path
- [ ] Create `components/mod.rs` to re-export global_search

**Files to create/modify:**
- `components/global_search/mod.rs` - Main component logic
- `components/global_search/view.rs` - View functions
- `components/mod.rs` - Re-export module
- Update imports in `app.rs`, `message.rs`, `update/workspace.rs`

## Phase 1: Debouncing & Async Search (Critical)
- [ ] Add 300ms debounce timer to `GlobalSearch` state
- [ ] Implement `iced::Task` for background search
- [ ] Add `SearchState` enum (Idle/Searching/Results/Error)
- [ ] Create async search function that returns `Task<Message>`

**Files to modify:**
- `components/global_search/mod.rs` - Add debounce state and async search
- `message.rs` - Add `SearchResultsReady` message variant
- `update/workspace.rs` - Handle async search completion

## Phase 2: Virtual Scrolling (High Impact)
- [ ] Replace full list render with virtual scrollable
- [ ] Only render 20 visible items at a time
- [ ] Implement scroll position tracking
- [ ] Add smooth scrolling behavior

**Files to modify:**
- `components/global_search/mod.rs` - Implement virtual scrolling logic
- `components/global_search/view.rs` - Update rendering to use viewport

## Phase 3: Result Caching (Optimization)
- [ ] Cache rendered `Element` for each result
- [ ] Implement `cached_element` field in `SearchResult`
- [ ] Invalidate cache on query changes
- [ ] Use `Arc` for shared result ownership

**Files to modify:**
- `components/global_search/mod.rs` - Add caching fields
- `components/global_search/view.rs` - Use cached elements when available

## Phase 4: String Interning (Micro-optimization)
- [ ] Add string interner for catalog types
- [ ] Intern display strings during search
- [ ] Reuse interned strings in rendering

**Files to modify:**
- `components/global_search/mod.rs` - Add string interner
- `search_index.rs` - Return interned strings

## Additional Improvements

### Accessibility Enhancements
- [ ] Add keyboard navigation (↑/↓ arrows, Enter to select)
- [ ] Screen reader support for search results
- [ ] Focus trapping within search modal
- [ ] ARIA attributes for result items

### UX Polish
- [ ] Loading spinner during async search
- [ ] Empty state illustration when no results
- [ ] Search query suggestions
- [ ] Result grouping by catalog type

### Code Quality
- [ ] Add comprehensive doc comments
- [ ] Property-based testing for search logic
- [ ] Benchmark tests for performance regression
- [ ] Error handling for failed searches

### Integration Points
- [ ] Global keyboard shortcut (Ctrl+K / Cmd+K)
- [ ] Search scope filtering (current file/all files)
- [ ] Recent search history
- [ ] Save/load search sessions

## Testing Plan
1. **Unit Tests**
   - Verify debounce delay works (no searches during typing)
   - Test async search completion handling
   - Validate virtual scrolling item calculation
   - Confirm cache invalidation on new queries

2. **Integration Tests**
   - Test search across 5000+ records
   - Verify keyboard navigation works
   - Confirm accessibility attributes present
   - Test error recovery scenarios

3. **Performance Tests**
   - Measure render time before/after caching
   - Test with 1000+ results to verify performance
   - Memory profiling during extended use
   - Frame rate monitoring during scrolling

4. **User Testing**
   - Blind keyboard navigation test
   - Screen reader compatibility check
   - Typical workflow timing (search → select → edit)

## Success Metrics

### Performance Targets
- **Typing**: <16ms frame time during input (60fps)
- **Search**: Results appear within 300ms of typing stop
- **Scrolling**: <5ms frame time with 10,000 results
- **Memory**: <5MB heap usage for search component
- **Startup**: Component init <10ms

### Quality Targets
- 100% test coverage for search logic
- 0 clippy warnings
- 0 unsafe code blocks
- Full keyboard operability
- WCAG 2.1 AA compliance

### UX Targets
- Search feels instantaneous
- No visible lag during typing
- Smooth 60fps scrolling
- Intuitive keyboard shortcuts
- Clear visual feedback

## Risk Assessment

### High Risk Items
- Virtual scrolling implementation complexity
- Async search race conditions
- Memory management with cached elements

### Mitigation Strategies
- Implement virtual scrolling incrementally
- Use Tokio mutex for async coordination
- Add memory pressure tests
- Comprehensive error handling

## Rollback Plan
1. Feature flag for new search implementation
2. Maintain old search code until validation complete
3. A/B testing capability
4. Quick revert procedure documented

## Documentation Requirements
- User-facing search shortcuts documentation
- Component API documentation
- Performance tuning guide
- Accessibility compliance notes
