# Menus

## Overview

- Use a menu to show a temporary set of actions. To show actions on screen at all times, use a toolbar instead
- Menus can open from many components, including icon buttons, split buttons, and text fields
- Context menus provide actions for a specific element, like an image or highlighted text, and usually open with a secondary click

*1 vertical menu with vibrant colors opens from a split button, and 1 vertical menu with a submenu.*

Availability & resources

M3 Expressive update

November 2025
Vertical menus were introduced with new shapes, color styles, selection states, and refined submenu motion. Gaps can be used for a more flexible layout on Android. More on M3 Expressive

Variants:
- Added vertical menus, recommended for new designs
- Baseline menu is still available  
Color styles: 
- Standard
- Vibrant

*A vertical menu using shape and vibrant color to show a selected state.*

Differences from M2

- Color: New color mappings and compatibility with dynamic color
- Variants: Dropdown menu and exposed dropdown menu are now both referred to as menu, since they differ only in the element which opens the menu surface

*Menu with gray color.*

*Menu with purple background and outline.*

## Specs

Variants

Vertical menus

Use vertical menus for a more expressive look and feel, including rounded corners, standard and vibrant color styles, more selection states, and submenu motion.

*2 vertical menus use shape and color to indicate selected state.*

Baseline variant

In M3 Expressive, baseline menu is still available to use, but doesn’t have the latest shapes, color styles, selection states, and motion. See baseline menu specs

*A baseline menu variant with square corners and standard colors.*

| | Variant
| M3
| M3 Expressive

| Vertical menus
| --
| Available

| Menu (baseline)
| Available
| Available

Configurations

Vertical menus layout

*2 menus: 1 standard, and 1 with a gap, creating groups.*

| | Category
| Configuration
| M3| M3 Expressive

| Color| Standard| Available| Available
| Vibrant| --| Available
| Layout| Standard| Available| Available
| Grouped| --| Available

Tokens & specs

Browse the component elements, attributes, tokens, and their values. Learn about design tokens

Anatomy

Vertical menus

*A diagram of a vertical menu.*

Color

Color values are implemented through design tokens. For designers, this means working with color values that correspond with tokens. In implementation, a color value will be a token that references a value. Learn more about design tokens

Menus have two color mappings:
- Standard: Surface-based
- Vibrant: Tertiary-based
These mappings provide options for lower or higher visual emphasis. Vibrant menus are more prominent so should be used sparingly.

*2 vertical menus: 1 with lower visual emphasis, and 1 vibrant menu with bold shades.*

Standard colors

*2 vertical menus with standard color roles mapped to 11 elements.*

Vibrant colors

*2 vertical menus with vibrant color roles mapped to 11 elements.*

States
States are visual representations used to communicate the status of a component or an interactive element. More on interaction states

Shape morphing in vertical menus creates an expressive active state. As focus moves between submenus, the corner shape changes to highlight the active menu. More on menu focus

*6 vertical menu states in light and dark themes.*

Measurements

*Vertical menu marked with spacing and padding measurements.*

Menu (baseline)

The baseline menu variant is available and continues to work in existing products. However, M3 expressive vertical menus are recommended for new designs.

Baseline tokens & specs

Browse the component elements, attributes, tokens, and their values. Learn about design tokens

Anatomy

*Diagram of 6 elements of a baseline menu.*

Color

*9 color roles of a baseline menu in light and dark themes.*

States

Default menu items

*Diagram numbering the 5 default states of a baseline menu.*

Selected menu items

*5 states of a selected baseline menu item.*

State specs are in the token module above

Measurements

*Diagram of a baseline menu’s padding, text alignment, height, and width.*

| | Attribute
| Value

| Container width
| 112dp min, 280dp max
| Corner radius
| 4dp
| Vertical label text alignment
| Center-aligned
| Horizontal label text alignment
| Start-aligned
| Left/right padding
| 12dp
| Left/right padding with-icon
| 12dp
| List item height
| 48dp
| Padding between elements within a list item
| 12dp
| Divider top/bottom padding
| 8dp
| Divider height
| 1dp
| Divider width
| Dynamic
| Leading/trailing icon size
| 24dp

Configurations

A baseline menu appears when a person interacts with a button, action, or other control. 
A few examples:
- Button
- Text field
- Icon button
- Selected text

*Examples of 4 baseline menu inputs.*

## Guidelines

*2 vertical menus, 1 with vibrant colors, 1 with standard colors and grouped items.*

Usage

Use a menu to show a temporary set of actions. To show actions on screen at all times, use a toolbar instead. 
A menu takes up less space than a set of radio buttons or chips. 

Color options
Menus have two color mappings:
- Standard: Surface-based, lower visual emphasis

- Vibrant: Tertiary-based, higher visual emphasis

Vibrant menus are more prominent, and should be used sparingly.

*Menu shows item “Line spacing” opening a submenu. In the second menu, “Custom 1.2” is selected with vibrant color.*

Opening menus
Menus temporarily appear in front of all other permanent UI elements.
A menu should open when a person:
- Selects an element, such as an icon, button, or text field

- Performs a specific action to trigger the menu, like right-click or press-and-hold

Use menus in situations that need extra actions, like: 
- Overflow menus

- Text field dropdown menus

- Select menus

- Context menus

*A grouped menu with Undo, Redo, Cut, Copy, and Paste options appear over highlighted text in an ebook.*

Menu groups
Vertical menu items can be grouped by adding a divider or small gap. Use groups to bundle similar actions together.  
Gaps and dividers guidelines

*2 vertical menus: a standard menu with no gap and a grouped menu with 1 gap.*

Context menus

Context menus provide a list of additional actions a person can take on an item. A secondary click, like a right-click on a mouse or a two-finger tap on a trackpad, opens a context menu.

*A context menu pops up from a newspaper link. The menu items are: Open in new window, Save link as, Copy address, and Inspect.*

Anatomy

*Diagram outlining 11 elements of a menu’s anatomy.*

Menu items

Menu items can include label text, leading icons, trailing icons, and keyboard commands. 
When a menu item can only be used under specific conditions, it should appear disabled rather than be removed.

*Menu shows 1 item that’s  disabled, “Redo”. The text color of the disabled item is lighter than the active items.*

Gaps & dividers (optional)
Gaps and dividers can be used to separate and group menu items.
Gaps
Use a gap to visually divide menu items into distinct groups. Gaps are more expressive than dividers and make the relationship between items clear.
- Avoid changing the size of the gap
- Limit the number of gaps in a menu to one or two
- Don’t use gaps in scrollable menus

*2 vertical menus with 5 items. A gap separates items into a group of 3 and group of 2.*

star
Note:
Gaps are not currently available on web

Dividers 
Dividers create a more subtle separation between items. Use a divider for:
- Scrollable menus
- Text fields with a dropdown menu, where a grouped treatment isn’t appropriate
On web, use a divider to separate menu items.

*A menu on a web interface with items separated by a divider line.*

Flexibility & slots

Menus have custom slots that support more flexible item layouts.
When creating a complicated menu, think of the menu item as a container with a swappable slot.
Slots work best with simple content such as:
- Images
- Progress indicators
- Color swatches

*A menu showing an undefined slot that could be used for a different element, such as an image.*

Slot accessibility

Use caution when adding slots to menus:
- Make sure the menu remains accessible
- Elements must follow the rules and interaction patterns of the menu component
- Keep the same menu item padding
- Targets should be 48x48dp or larger
Don't add buttons, switches, or other direct actions into the menu item. Nested elements should only perform one action. Adding multiple actions can break keyboard navigation and screen reader functionality.
More on required accessibility guidelines

*1 diagram and 1 menu showing icons in each item’s leading slot.*

Placement

A menu is positioned relative to the window edge. It typically appears below, next to, or in front of the element that generates it.
If a menu is in a position to be cut off, it should automatically reposition to appear to the left, right, or above the element that generates it.

*6 abstract shapes showing how a menu can extend from the edge of the screen.*

Submenus
Submenus should open next to the parent menu item without overlapping it.
Submenus are best used on large screens where there's space. See adaptive guidance for alternatives on mobile.

*A submenu opens to the right of its parent menu item, and doesn’t cover it. A selected submenu item includes a checkmark and vibrant highlight.*

star
Note:
Submenus are not currently available on Jetpack Compose

Adaptive design

Compact breakpoints
Consider adapting menus into bottom sheets on small screens. They have more space to display additional items and longer labels.

*A bottom sheet shows longer labels and improved readability on a compact window.*

Other breakpoints
On medium and expanded windows, menus are most effective as they appear in context with the content. On larger screens, menus can also display more items, and can use submenus to organize complex sets of options.

*A menu with vibrant color on a mid-size screen, with the same elements as a bottom sheet.*

Behavior

Appearing
A menu can appear when a person interacts with an element on the page, like a button, text field, filter chip, or highlighted text.
A menu’s position on screen affects where and how it appears. If opened at the top of the screen, it expands downwards to avoid being cropped.

*A menu activated at the top of the screen expands downwards, then a menu opened at the bottom of the screen expands upward.*

*A menu expands downward from the top of the screen, appearing below a split button.*

*A menu expands both above and below a line of selected text, separated by a gap.*

*Selecting the “Phone type” text field reveals a menu with multiple options: Business, Mobile (selected), and Home.*

*A filter chip for “Cycling” in a map UI reveals more menus items: Running, Walking, and Hiking.*

Motion
Menus use an enter and exit transition. This animation creates a relationship between the menu and the element that generates it.
When a menu expands, the trigger element becomes pressed. When an item is selected, a ripple appears on touch.

*An animation for entering a new contact’s address. The state selection menu expands and the state California is selected.*

In dense products, such as on desktop, menus can open instantly to reduce motion.

*A menu for changing a font type opens instantly on a desktop UI.*

Filtering
A menu can include a text field to filter options. This pattern is also known as autocomplete. 
As someone types, the list of menu options filters to show relevant results. This helps people quickly find the right option from a long list. 
Menu items ease into their new position as the menu is filtered.

*An animation showing a text field being typed into. As text is added, the list of menu items below filters down to show only matching options.*

Scrolling
Menus can scroll when all menu items can’t display at once. In this state, menus show a persistent scrollbar.
Don’t use gaps if a menu scrolls; this is currently unsupported.

*A font menu on a document shows a scrollbar to access font options not currently visible.*

Selecting
When a menu is opened, the corresponding button or icon button should remain the same visually, with the addition of a pressed state.
This should happen even when opening from a keyboard shortcut.

*The overflow icon remains the same, even after the menu is opened.*

Single- and multi-select menus
Menus can allow either single-select or multi-select actions:
- Single-select menus can have one item selected at a time. When a new item is selected, the previously selected item is automatically unselected.
- Multi-select menus can have many selected items. They stay open until the person dismisses the menu.
More on selection accessibility requirements

*1 menu for dietary options shows a single selection, Vegan. Another menu shows Vegan and Nut-free selections at the same time.*

Focus
When a menu has multiple submenus, focus follows the current hovered or focused submenu. 
Shape morphing
As a person moves from one submenu to the next, the corners of the focused submenu become more rounded, while the unfocused submenu becomes less rounded. This adds a dynamic quality to menu interactions.

*On a submenu next to a main menu, a selected item’s corner shape expands for added emphasis.*

Density

On web only, density levels control the spacing between elements. Increasing density decreases the top and bottom padding. More on layout density

*4 menus becoming increasingly dense and compressed.*

## Accessibility

Use cases

People should be able to do the following using assistive technology:
- Navigate to, open, and close a menu
- Navigate between and select menu items

Interaction & style

Menu items need certain cues to clearly show when they're selected: 
- By default, menu items change shape and color when selected
- The default color contrast is 3:1 between selected and unselected menu items
- It's recommended to include another visual cue, like a checkmark

*A state dropdown menu with the selected item Alaska highlighted in a vibrant color, with a checkmark icon.*

Flexibility & slots

Use caution when adding slots to menus:
- Make sure the menu remains accessible
- Elements must follow the rules and interaction patterns of the menu component
- Keep the same menu item padding
- Targets should be 48x48dp or larger
Don't add buttons, switches, or other direct actions into the menu item. Nested elements should only perform one action. Adding multiple actions can break keyboard navigation and screen reader functionality.
More on slots in menus

*1 diagram and 1 menu showing icons in each item’s leading slot.*

Focus

Initial focus
When a menu opens, focus should be placed on the first menu item. This allows people using a keyboard or other assistive technologies to begin navigating the menu immediately.
Exiting a menu
People expect to exit a menu by:
- Selecting an option
- Tapping Escape or outside of the menu 
- Using the system back button
Where focus is placed after closing the menu depends on the app.

*4 common keyboard navigation methods for menus on Android and web.*

Keyboard navigation

| | Keys
| Actions

| Tab| Focus lands on menu
| Space or Enter
| For closed menus: Opens menu or submenu
For open menus: Selects a menu item

| Up and Down arrows| For closed menus: Opens menu 
For open menus: Moves focus to the next item

| Left and Right arrows| Opens or closes a submenu
| Letters| Focus moves to the next menu item starting with letter
| Escape| Closes menu

Interactability

Disabled menu items can receive focus but aren't selectable.
Dividers and gaps can't receive focus.

*A disabled menu item “Share” is in focus.*

*A divider with focus.*

On web, submenus have a magic triangle behavior for more intuitive mouse navigation. 
This keeps the submenu open when the mouse moves within a triangular area between the triggering item and the submenu.
More on submenus

*A menu opens a submenu, and the pointer moves in a triangular area to the submenu item “Custom font family” without closing the menu.*

Labeling elements

Accessibility labels are used with assistive technology devices like screen readers. 
The accessibility label should be the same as the menu item text.
The role is dependent on platform.

*A “Preview” menu item has an accessibility label of ”preview”.*

| | Element
| A11y label
| Role (Web)
| Role (Android Views)

| Role (Jetpack Compose)

| Menu item text| Preview| Menu item| Generic actionable element| Generic actionable element

For menu items with text and an icon, the icon’s accessibility label should be marked as decorative to avoid redundant verbalizations.

*A menu item icon of an eye next to the word “preview” has a note of “Decorative.”*
