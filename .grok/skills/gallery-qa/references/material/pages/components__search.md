# Search

## Overview

- Use search for navigating a product with queries

- A search bar can include a leading search icon, hinted search text, and optional trailing icons

- Search can display suggested keywords or phrases as a person types

- A search bar displays search suggestions or results in a list

- Use a search app bar to provide an emphasized, global entry-point

*Mobile UI shows a person typing into an email search bar. It expands to show a list of results.*

Availability & resources

M3 Expressive update

Search has a new visual style, motion, and more flexibility for trailing icons. More on M3 Expressive

February 2025 
Naming
- Search bar and search view are now collectively named search

Configurations
- Styles: Search can be contained (recommended) or divided

- Gaps can separate results into groups

Motion
- The search bar grows wider when focused

Supported platforms:
- Jetpack Compose

*A recipe search with “Search recipes” hinted text, “Mexican dishes” is entered, then results appear in a list.*

Differences from M2 to M3 baseline

- Color: New color mappings and compatibility with dynamic color

- Elevation: Lower elevation and no shadow by default

- Name: Search was formerly known as open search bar

- Variants: Two official variants of search components: search bar and search view

*M2 open search bar.*

*M3 search bar.*

## Specs

Variants

*Typing “Ping” into a search bar reveals a list of email results.*

| | Variant
| M3
| M3 Expressive

| Search
| Available
| Available

Configurations

Style

Search comes in two styles:
- Contained: Has an expressive look and feel. It uses a filled container to separate a search bar from a list of suggestions or results

- Divided (baseline): Doesn’t have the latest visual style, motion, or flexibility

*An email inbox search bar in a contained style.*

*An email inbox search bar in a divided style.*

Layout

Search suggestions and results appear in customizable lists, with two layout options: full-screen and docked. More on search layouts

*Full-screen search results with a search bar in the contained style.*

*Docked search results with a search bar in the contained style.*

*Full-screen search results with a search bar in the divided style.*

*Docked search results with a search bar in the divided style.*

| | Category
| Configuration
| M3
| M3 Expressive

| Style
| Contained
| --
| Available

| | Divided
| Available
| Not recommended. Use contained.

| Layout
| Docked, full-screen
| Available
| Available

Tokens & specs
Use the table's menu to select a token set. The search bar set only contains tokens for the unfocused search bar. The search view set contains all other tokens when interacting with search, including all styles and layouts. Learn more about design tokens

Anatomy

Search includes a search bar and a container for suggestions and results. The container is empty by default. Use the list component to add content. In the divided (baseline) style, a divider separates the search bar and results.

*6 elements of search.*

Examples
- With avatar

- With one trailing icon button

- With two trailing icon buttons

- With trailing icon button and avatar

*4 search bars with different trailing elements.*

Color

Color values are implemented through design tokens. For designers, this means working with color values that correspond with tokens. In implementation, a color value will be a token that references a value.

Full-screen layout

*6 full-screen search color roles in light and dark themes.*

Docked layout

*6 docked search color roles in light and dark themes.*

States

States are visual representations used to communicate the status of a component or an interactive element. In focused search, individual elements maintain their own interaction states. Learn more about interaction states

Search bar

*4 search bar states in light and dark mode.*

Search suggestions & results

Search includes a container for suggestions and results. The container is empty by default. Use the list component to add content.

*4 search result states in light and dark mode.*

Measurements

Search bar

*Search bar with leading and trailing icon size and padding measurements.*

*Search bar with trailing avatar size and padding measurements.*

In M3 Expressive, the search bar expands when focused. The margins change from 24dp to 12dp.

*Unfocused search bar margins of 24dp.*

*Focused search bar margins of 12dp.*

| | Element
| Attribute
| Value

| Container
| Width
| Min: 360dp, max: 720dp

| Height
| 56dp

| Label alignment
| Start-aligned

| Leading padding
| Unfocused: 24dp, focused: 12dp

| Trailing padding
| Unfocused: 24dp, focused: 12dp

| Leading icon and label padding (from tap target)
| 4dp

| Label and trailing icon padding (from tap target)
| 4dp

| Avatar
| Size
| 30dp

Focused search

Contained style

*Full-screen layout size and padding measurements in contained style.*

*Docked layout size and padding measurements in contained style.*

| | Element
| Attribute
| Value

| Full-screen container
| Width
| Full width

| Height
| Full height

| Docked container
| Width
| Min: 360dp, max: 720dp

| Height
| Min: 240dp, max: 2/3 of screen height

| Search bar container
| Height
| 56dp

| Label alignment
| Start-aligned

| Leading padding
| 16dp

| Trailing padding
| 16dp

| Leading icon and label padding (from tap target)
| 4dp

| Leading icon and label padding (from tap target)
| 4dp

Divided style

*Full-screen layout size and padding measurements in divided style.*

*Docked layout size and padding measurements in divided style.*

## Guidelines

*A mobile UI search with hinted text “Search recipes”, “Mexican dishes” is entered, and a list of recipe results appear.*

Usage

Search helps people find information quickly.

Use search for products with many items to manage, such as files or messages.

*Mobile UI shows a search bar at the top of a message inbox.*

Different ways to search

The search entry point is dependent on a product’s needs, and should be easy to find:
- Search bar: Use to search contents in a specific view, like Search your messages

- Search app bar: Use this app bar variant when search is the primary, global function

- Search icon button: Use when search is a secondary action or not the main focus

*A mobile app with a search bar below the page title.*

*A mobile app with a search app bar.*

*A mobile app with a magnifying glass icon on the leading side of the app bar.*

Focused search
When a search entry point is selected, it opens focused search.  
- Search suggestions can appear before text is entered

- Search results can show as someone is typing or after a search is executed

- Individual elements maintain their own interaction states when search is focused

More on search states

*Focused search with a list of suggestions on a mobile screen.*

If search is the primary action, focused search can be a standalone destination reached from a navigation bar.

*Focused search on a mobile screen with a list of suggested contacts.*

Search suggestions & results
Search suggestions and results both appear in a list component by default.

To help people find information quickly, consider adding variety and context, such as:
- Leading icons related to suggestions

- Category labels, like Recent, Contacts, or Suggestions

- Avatars or other high-priority items

- Filter chips to narrow down results

*Search with suggestions organized in a column, ending with a row of 5 contact avatars with names.*

Gaps
Use gaps to separate a list of suggestions or results into groups.

More on using gaps in lists

*A gap separates the location and calendar list items from people and pets avatars.*

Placement

A search bar is typically placed at the top of a screen to remain prominent and accessible. Its location depends on whether search is the primary focus of a product or a secondary action.

*Mobile UI with a search bar directly below a Settings headline.*

*Mobile UI with a search bar centered at the top of the screen, above a row of Favorites avatars.*

*A photos app with a search icon.*

Focused search layouts
When focused, search suggestions and results appear in a list below the search bar.  

There are two layout options:
- Docked opens a list below the search bar, with a scrim covering main content

- Full-screen expands to fill the screen

More on adaptive design

*Tablet UI shows a list of search results docked below the search bar.*

*Mobile UI shows a list of search results filling the screen.*

Anatomy

*6 elements of search.*

Search bar container
In the contained style, the search bar container remains the same shape in both the unfocused and focused states. Avoid changing the container behavior.

The container’s margins should be:
- Unfocused: 24dp

- Focused: 12dp

In the divided (baseline) style, a divider separates the search bar and results.

*Side-by-side comparison of a search container in unfocused and focused states.*

Container color
Search bars use the surface container high color role. This role applies when the screen background is white or a tonal surface color, ensuring the container has clear contrast.

*2 mobile UIs show search bars on white and tonal backgrounds.*

Avoid using a surface container high color on a surface container background. This can cause the search bar to blend in, making it difficult for people to find.

To ensure proper contrast, use surface container roles that are more than one step apart.

*A “surface container high” search bar on a “surface container” background.*

Icons & icon buttons

Leading icons
The leading side of a search bar should include either:
- A navigational icon button, such as a menu or arrow

- A non-functional search icon

*A search bar on a tablet screen contains a non-functional search icon and a trailing avatar.*

Trailing icons
A search bar should have one or two trailing icons or icon buttons.

Trailing actions can include:
- Additional modes of searching like voice search

- A separate high-level action such as current location or profile

- An overflow menu

- A decorative search icon

*A search bar with 2 trailing icon buttons: a microphone and an overflow menu.*

*A search bar with a trailing microphone icon and avatar.*

*Focused search with a trailing x icon to clear input text.*

Text

Hinted search text
Provide a short description of the information people can search, like Search replies or Search your messages.

Input text
When a person starts typing, the hinted text is replaced with the input text.

*A search bar labeled “Search replies”. “Peanut is entered and “Quick results” appear.*

Adaptive design

The search bar position and alignment should scale with the layout, and stay close to the searchable content.

In most cases, a search bar should:
- Stay in its pane and scale in width accordingly

- Internal elements anchor to the left and right as the parent container scales

More on applying layout

*A search bar keeps its layout region and scales with different window sizes and layouts.*

Focused search
When focused, search can switch between showing suggestions or results in a:
- Docked layout: Best for medium and expanded windows

- Full-screen layout: Default for compact breakpoints

More on search layouts

*Search suggestions in docked and full screen layouts.*

Search suggestions or results should swap from full-screen in compact windows to docked in larger breakpoints.

*Animation shows search suggestions adapting from full-screen on mobile to a docked layout as the window size increases.*

Behavior

Focused search

When a search bar is selected, search becomes focused and can:
- Show historical suggestions before typing

- Show suggestions or results as someone is typing

- Wait to show suggestions or results until a search is queried 

The back icon releases focus, dismisses any suggestions or results, and returns the search bar to its original state.

*When a search bar is tapped, it becomes focused, and suggestions appear in a list.*

*A person searches a photo app. The back icon returns the search bar to its original state.*

Scroll
Depending on needs, a search bar can:
- Scroll away with content, then reappear when a person begins scrolling up

- Remain fixed at the top of the screen

*Scrolling up hides the search bar. It reappears when scrolling down.*

Search results
To execute a search, a person can:
- Type a query and press Enter

- Select a suggestion or result without querying a search

Search results appear in a list below the bar, and scroll beneath the bar.   

For accessibility, focused search needs a clear status indicator that it’s searching content, like a search icon or Results label. More on search accessibility

*“Peanut” is the entered search query and the first suggestion in the list.*

When search results are queried, the input text should remain visible, but not in focus.

*“Pla” is entered into the search bar, “Plants” is suggested and selected.*

Predictive back
On Android, predictive back allows a person to swipe left or right on search. 
- Search detaches from the screen edge to signal the full-screen layout will minimize

- The previous screen is revealed in a preview

More predictive back design guidance

*Swiping left on search causes the Android screen to scale left.*

## Accessibility

Use cases

People should be able to use assistive technology to:
- Navigate to and focus on a search bar

- View the hinted search text or persistent label

- Input text and complete a search

- Interact with a list of search suggestions and results

- Clear the input text

Interaction & style

Autosuggest
When search suggestions and results appear, the screen reader must announce the change. This lets people know list items are available for selection.

*Hinted search text and autocomplete results on a mobile screen.*

Initial focus

Initial focus lands on the first interactive element. This is often a leading icon button or text field. A leading icon button usually activates search directly or opens a navigation component.

*Search bar with a focused leading icon.*

*Search bar with no leading icon. The text field is focused.*

Keyboard navigation

| | Keys
| Actions

| Tab or Shift + Tab
| Navigate between interactive elements

| Space or Enter
| Activate the search text field for input

| Arrows
| Navigate between search result items

Labeling elements

The hinted search text should be used as the accessibility label describing the search bar.  

The role for the input field should be:
- Android: Text field

- iOS: Search field

*Search bar with “Label: Search messages” and “Role: Text field”.*

Leading and trailing icon buttons should be labeled according to their accessibility guidance.

*A search bar with accessibility labels for its leading icon button and trailing avatar.*

Search suggestions and results use the list component. Screen readers automatically announce the results as a list.

For accessibility labels, follow the list accessibility guidelines.

*A search bar on mobile, showing search results in a list.*
