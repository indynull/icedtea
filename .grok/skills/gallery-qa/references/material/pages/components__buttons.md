# Buttons

## Overview

- Two variants: default and toggle

- Can contain an optional leading icon

- Five color options: elevated, filled, tonal, outlined, and text

- Five size recommendations: extra small, small, medium, large, and extra large

- Two shape options: round and square

- Keep labels concise and use sentence case

*5 variants of buttons.*

Availability & resources

M3 Expressive update

May 2025
Buttons now have a wider variety of shapes and sizes, toggle functionality, and can change shape when selected. More on M3 Expressive

Variants and naming:
- Default and toggle (selection)

- Color styles are now configurations (elevated, filled, tonal, outlined, text)

Shapes: 
- Round and square

- Shape morphs when pressed

- Shape morphs when selected

Sizes:
- Extra small

- Small (existing, default)

- Medium

- Large

- Extra large

New padding for small buttons:
- 16dp (recommended to match padding of new sizes)

- 24dp (no longer recommended)

*4 button changes in the expressive update.*

Differences from M2

- Color: New color mappings and compatibility with dynamic color. Icons and labels now share the same color. Neutral text button is no longer recommended.

- Icons: Standard size for leading and trailing icons is now 20dp

- Shape: Fully-rounded corner radius and additional height options

*Rectangular M2 buttons.*

*Round-cornered M3 buttons.*

## Specs

Variants

*Diagram comparing buttons with toggle buttons.*

| | Variant
| M3
| M3 Expressive

| Default
| Available
| Available

| Toggle (selection)
| --
| Available

Configurations

*Diagram showing configurations of buttons.*

| | Category
| Configuration
| M3
| M3 Expressive

| Size
| Small (default)
| Available
| Available

| XS, M, L, XL
| --
| Available

| Shape
| Round (default)
| Available
| Available

| Square
| --
| Available

| Color
| Elevated, filled (default), tonal, outlined, text
| Available
| Available

| Small button padding
| 24dp
| Available
| Not recommended.
Use 16dp

| 16dp
| --
| Available

Tokens & specs
Use the table's menu to select a token set. Button token sets are separated into common tokens, color, and size. View baseline tokens

Anatomy

*Diagram labeling 3 parts of a button.*

Color

Color values are implemented through design tokens. For designers, this means working with color values that correspond with tokens. In implementation, a color value will be a token that references a value.
- There are five built-in button color styles: elevated, filled, tonal, outlined, and text

- The default and toggle buttons use different colors

- Toggle buttons don’t use the text style

star
Note:
These color roles were chosen to create design coherence and familiarity. Other color roles can be used as long as the container and text have a 3:1 contrast ratio. For example, tertiary and on tertiary.

*Diagram shows dark and light color schemes for buttons.*

| |
| 1. Default
| 2. Toggle unselected
| 3. Toggle selected

| Elevated container
Elevated icon & label
| Surface container low
Primary
| Surface container low
Primary
| Primary
On primary

| Filled container
Filled icon & label
| Primary
On primary
| Surface container
On surface variant
| Primary
On primary

| Tonal container
Tonal icon & label
| Secondary container
On secondary container
| Secondary container
On secondary container
| Secondary
On secondary

| Outlined container
Outlined icon & label
| Outline variant (outline)
On surface variant
| Outline variant (outline)
On surface variant
| Inverse surface
Inverse on surface

| Text icon & label
| Primary
| --
| --

High contrast

High contrast mode is an accessibility feature that aims to maximize legibility by using a limited color palette, like black and white. Focus rings are shown when buttons are focused.

*High contrast mode for each variant and style of button.*

States

States are visual representations used to communicate the status of a component or interactive element.

Elevated button states
The elevated button style has an elevation of 1 by default and 0 when disabled.

Default

*Elevated button states.*

Toggle

*Toggle elevated button states.*

Filled button states

Default

*Filled button states.*

Toggle

*Toggle filled button states.*

Tonal button states

Default

*Tonal button states.*

Toggle

*Toggle tonal button states.*

Outlined button states

The outlined button’s container fill is invisible at rest, but the opacity and state layers behave the same as other button styles when disabled, hovered, focused, or pressed.

Default

*Outlined button states.*

Toggle

*Outlined button states.*

Text button style states

The text button’s container is invisible at rest, but the opacity and state layers behave the same as other button styles when disabled, hovered, focused, or pressed. There is no toggle text button.

*Default text button style states.*

Shape morph

Pressed state
When pressed, buttons can morph to become more square. Both round and square buttons should have the same pressed shape.
The corner radius value differs for each button size. See full button corner measurements

*Shape changes of a button.*

When selected
In addition to changing shape when pressed, toggle buttons also change the resting shape from round (unselected) to square (selected). 
If the resting unselected shape is square, the selected shape should be round.

*Shape changes of a toggle button.*

Measurements

*Diagram of measurements of all button sizes.*

Target areas

Extra small and small icon buttons must have a target size of 48x48dp or larger to be accessible.

*Diagram of small button target areas.*

Corner sizes

*Diagram of corner radii of buttons.*

| |
| XS
| S
| M
| L
| XL

| A. Round button| Full| Full| Full| Full| Full
| B. Square button| 12dp| 12dp| 16dp| 28dp| 28dp
| C. Pressed state| 8dp| 8dp| 12dp| 16dp| 16dp

Baseline tokens

Use the table's menu to switch token sets. The baseline button token sets are organized by color.

## Guidelines

*Buttons in various shapes and sizes.*

Usage

Buttons communicate actions that people can take. They are typically placed throughout the UI, in places like:
- Dialogs

- Modal windows

- Forms

- Cards

- Toolbars

They can also be placed within standard button groups.

*Video call app with prominent filled button to join and end a call.*

Buttons are just one option for representing actions in a product and shouldn’t be overused. Too many buttons on a screen can disrupt the visual hierarchy.

Consider placing additional actions in a navigation rail, set of chips, text links, or icon buttons.

*1 button placed on bottom right of screen.*

*3 buttons side by side on bottom of screen.*

*Filled button on menu screen.*

*Filled button as wide as layout grid.*

*Filled button with label text overflowing the container.*

*Diagram of button styles and toggle behaviors.*

A button group is a collection of buttons that relate to each other and can respond to one another. Both buttons and icon buttons can be used inside a button group.

In some cases, there are primary and secondary actions within a button group. Buttons with primary actions should have a higher visual emphasis through size, color, or shape.
More on button groups

*Audio app with play, next, and back buttons.*

Toggle buttons

Toggle buttons should be used for binary selections, such as Save or Favorite. When toggle buttons are pressed, they can change color, shape, and labels.
Toggle buttons should use an outlined icon when unselected, and a filled version of the icon when selected. If a filled version doesn’t exist, increase the weight instead.
By default, toggle buttons change from round to square when selected.

*Toggle “stop” button in timer app.*

If the label changes on selected or unselected states, be mindful of the character count. Changing the label significantly is disruptive to the user and the page layout.

*Toggleable “start” and “reset” buttons.*

*Toggleable “start” and “reset back to beginning” buttons.*

Anatomy

*3 parts of a button.*

Label text
Label text is the most important element of a button. It describes the action that will occur if someone taps a button. It should be very brief, ideally 1–3 words.
Use sentence case, which only capitalizes the first word and proper nouns. This allows the text to distinguish proper nouns, for example: Book with Flights, not BOOK WITH FLIGHTS.
Don’t truncate or wrap label text. It should always be fully visible on a single line.

*Button with label text “See all recipes.”*

*Button with wrapped label.*

Buttons with the outlined and text color style depend on the colors to be recognizable from other text and elements. Use caution when putting these buttons next to visually similar elements, such as chips or large text.

*Chips next to an outlined button, highlighting their similarities.*

Container
Button containers hold the label text and optional icon. Buttons with the text color style have a visible container only when hovered, focused, or pressed.
Buttons with a round shape have containers with fully rounded corners.

*Round button.*

Buttons with a square shape have containers with more subtle rounding that changes based on button size.

*Square buttons with different radii.*

*Button with the label text “Edit playlist” within the container.*

*Button with text larger than its container.*

Icon (optional)
Icons visually communicate the button’s action and help draw attention. They should be placed on the leading side of the button, before the label text.

*Filled button with the icon to the left of the label in a left-to-right language.*

*Filled button with the icon to the right of the label in a right-to-left language.*

*Button with shopping cart icon and text label “Add to cart”.*

*Button with Plus icon vertically above the text label “Add to watch list”.*

*Button with two icons.*

Color styles

Elevated style

The elevated button style is the same as the tonal button, but with a shadow. 
To avoid overusing shadows, use the elevated style only when absolutely necessary, such as when the button requires visual separation from a visually prominent background.

*Elevated button on a scrim background.*

Buttons at higher elevations typically have more emphasis in a design, and should be used sparingly. For high emphasis, consider the filled style instead.

*Elevated button in a shopping experience.*

Filled style

The filled button style has the most visual impact after the FAB, and should be used for important, final actions that complete a flow, like Save, Join now, or Confirm.

*Filled button reading “Make payment.”*

Since they have such strong emphasis, the filled style should be used sparingly, ideally for only one action on a page.
In some cases, filled buttons can use tertiary colors.

*Filled “pause” button in a music app.*

Tonal style

The tonal button style is useful in contexts where a lower-priority button requires slightly more emphasis than an outline would give, such as Next in an onboarding flow. Tonal buttons use the secondary color mapping.

*Shopping app with 2 tonal-style filled buttons.*

Outlined style

The outlined style is ideal for medium-emphasis buttons which contain actions that are important, but aren’t the primary action in a product.
Outlined buttons pair well with filled buttons to indicate alternative, secondary actions.

*Outlined buttons for less important actions, including a back button and a button that reads “Next movie.”*

Outlined buttons display a stroke around the button container, and have no fill by default. 

They should be placed on simple backgrounds, not visually prominent backgrounds such as images or videos.

*Outlined button for “add to cart” in shopping app.*

*Outlined button labeled Add to calendar on a pink/purple background.*

*2 photos, each with an outlined button with a custom fill.*

Text style

The text button style should be used for the lowest priority actions, especially when presenting multiple options.
They should be placed on simple backgrounds, not visually prominent backgrounds such as images or videos. The container isn’t visible until someone interacts with the button.
Don’t underline the text button. Use hyperlinked body text instead to emphasize links. More on hyperlinks

*Example calendar screen with 2 text buttons and 1 split button.*

Text buttons are often placed within components such as cards, dialogs, and snackbars. Since text buttons don’t have a visible container in their default state, they don’t distract from nearby content.
However, since there’s no container, the label text color must always be recognizable from non-button text and elements.

*Text button labeled “Retry” in a snackbar.*

*Text button labeled “View album” on an album cover background.*

In cards, text buttons help maintain an emphasis on card content.

*Text button labeled “Learn more” in an information card about sourdough bread.*

Dialogs use text buttons because the absence of a container helps unify the action with the dialog text.
Align text buttons to the trailing edge of dialogs, on the right for left-to-right languages and on the left for right-to-left languages.

*Modal dialog with the title “Subscribe to our newsletter?” and trailing buttons “Cancel” and “Subscribe”.*

Adaptive design

Resizing

When scaling layouts for large screen devices, buttons can adapt their visual presentation, size, alignment, and arrangement to fit different contexts and user needs.

Choose the best button position based on screen size.

*Flights app in compact screen with buttons below flight information.*

*Flights app in large screen with buttons to the left of flight information.*

The icon and label text in a button stay centered and grouped as the button's width changes.

*2 buttons with horizontally centered text labels.*

*1 button with centered text label, 1 button with icon and label aligned to opposite edges.*

Buttons can be customized to change size and scaling behavior across different breakpoints. 
To avoid creating very long buttons in large windows, constrain button width or place buttons beside other elements.

*Button width is over-stretched with screen width.*

Presentation

The size and placement of buttons can change as parent containers, such as cards, adapt for larger screens. 
Keep items, including buttons, in the same order between large and small screens to provide a consistent experience for screen readers and keyboard navigation.

*2 buttons scaling to accommodate different device sizes.*

## Accessibility

Use cases
People should be able to do the following with assistive technology: 
- Use a button to perform an action
- Navigate to and activate a button

Interaction & style

Color contrast
Enabled buttons need a 3:1 contrast ratio with the background to meet accessibility best practices. 
This is measured from the container for elevated, filled, and tonal button styles, and the label text for outlined and text button styles.

*Diagram of color contrast ratios for buttons.*

200% text size
Avoid excessive text wrapping or truncation by choosing concise strings. 
On Android, button labels should be kept concise enough to fit within two lines after the text size is increased to 200%. If a button label exceeds this limit and gets truncated, provide an alternative way to access the full content in a single tap.

*200% text size on a mobile screen. The overly long button text wraps to a second line: “Download playlist for offline access”.*

Rapid clicks
On the web, you can use a modified motion curve to avoid resonant effects from overlapping animations. This provides a smoother experience for interactions where you anticipate multiple clicks or taps in succession.

*A media player where the “next track” button is clicked rapidly, and is transformed with a smooth motion effect.*

Keyboard navigation

| | Keys| Actions

| Tab| Navigate to a button
| Space or Enter| Activate a button

Labeling elements

The accessibility label for a button should match the visible label text on the button such as Done, Send, or Reply. 
It can contain extra contextual information if necessary.

*Accessibility tags for a text-only button.*
