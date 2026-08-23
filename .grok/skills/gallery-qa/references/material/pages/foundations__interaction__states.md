# States

States show the interaction status of a component or UI element

## Overview

- States have two visual indicators to ensure accessibility
- States can be combined, such as selection and hover
- Apply states consistently across components

Resources

| | Type| Link| Status

| Design| Design Kit| Available

1. Enabled
An enabled state communicates an interactive component or element.
2. Disabled
A disabled state communicates an inoperable component or element.

*Enabled button has a strong contrast between container and text.*

*Disabled button has low contrast grey text on grey container.*

3. Hover
A hover state communicates when a user has placed a cursor above an interactive element.
4. Focused
A focused state communicates when a user has highlighted an element, using an input method such as a keyboard or voice.

*Cursor positioned over a button.*

*The focused button has a strong contrast between container and text.*

5. Pressed
A pressed state communicates a user tap.
6. Dragged
A dragged state communicates when a user presses and moves an element.

*The pressed button has a strong contrast between container and text.*

*Dragged chip*

## State layers

State layers

A state layer is a semi-transparent covering on an element that indicates its state. State layers provide a systematic approach to visualizing states by using opacity. A layer can be applied to an entire element or in a circular shape and only one state layer can be applied at a given time.

To transition from an enabled style to a stateful style requires the addition of a state layer.
The state layer is an overlay with a fixed opacity for each state and uses the same color as the content. 
For example, if the enabled style uses the secondary container color for the container and on secondary container for content, the state layer will be an overlay using the on secondary container color.  
If the enabled style uses the surface color for the container and the primary color role for content, then the state layer will be an overlay using the primary color.

*State layer sandwiched between the container and content.*

The size of state layers is 40dp while the interactive target size is 48dp.

*Interaction target is larger than the state layer.*

On colors
By default, a component’s state layer color is derived from the color of its content, either the color of an icon or label text if no icon is present.
An on color is a color role used by the content. Each container color has its own corresponding on color. For example, if a container color is secondary container, the content will use the on secondary container color role.

*State layer sandwiched between  the content and container.*

State layer tokens & values
The state layer uses a fixed percentage for the opacity for each state. A state layer uses the color used by content (usually the on color) and the percentage opacity for its respective state.

*The opacity values of four overlay states.*

Activated states
Unlike hover, focus, pressed, and dragged states that use state layers, the container and content of activated components change color directly. 
Activated components use the secondary container color for the component container and change the content color to on secondary container.

*Activated flights container with color changed compared to containers for hotels, and restaurants both not in activated state.*

## Applying states

Enabled

An enabled state communicates an interactive component or element. Enabled states use the default styling for each interactive component.

*Enabled state of 4 components.*

Disabled

A disabled state communicates when a component or element isn’t interactive. This state is visually communicated through color changes and reduced elevation.
Disabled states don't need to meet Material's contrast requirements.

*Low opacity solitary button labeled disabled, indicates a  disabled/inoperable state.*

Disabled states are inherited by action, selection, and input components:
- Buttons
- Cards
- Checkboxes
- Chips
- List items
- Radio buttons
- Switches
- Text fields

*Inoperable state of 4 components.*

Disabled states aren't inherited by communication, containment, navigation, and some actions components: 
- App bars
- Badges
- Dialogs
- Floating action buttons (FABs)
- Menus
- Navigation bar, drawer, and rail
- Sheets
- Tabs
- Tooltips

*Screen erroneously showing edit FAB in inoperable state, though the edit function is unavailable.*

Behavior
Disabled components can’t be focused, dragged, or pressed, and they don’t change state when tapped or hovered over.

*A cursor moves over and clicks on an disabled/inoperable button and the button doesn’t change.*

There can be any number of disabled states in a layout.

*Disabled components on a screen.*

Hover

Hover states are initiated by the user pausing over an interactive element using a cursor.
The lower-emphasis surface overlay for hover states can be applied to the entire component, elements within a component, or as a circular shape over part of the component.

*Cursor moves toward button which reads “enabled” and when the cursor touches the button text changes to “hovered.”*

Hover states are inherited by action, selection, and input components:
- Buttons
- Cards
- Checkbox
- Chips
- Date and time pickers
- List items
- Slider
- Switch
- Text fields

*Hover state of 4 components.*

Hover states aren’t inherited by communication, containment, or navigation components: 
- App bars
- Badges
- Dialogs
- Menus
- Navigation bar, drawer, and rail
- Sheets
- Tabs

*Mobile screen with the whole  app bar wrongly in hover state.*

Behavior
Hover states are initiated by the user pausing over an interactive element using a cursor.

*Button’s text“Enabled”  changes to“Hovered” when cursor moves over the button.*

Hover states can be combined with focused, activated, selected, or pressed states.

*Filter chip text matches state as it's unselected, hovered, and selected by a cursor.*

There can only be one hover state at a time in a layout.

*Hover state moves from one card to another with cursor movement.*

Focused

A focused state communicates when a user has highlighted an element using a keyboard or voice. Focus states apply to all interactive components.
The higher-emphasis surface overlay for focused states can be applied to the entire component, elements within a component, or as a circular shape over part of the component.

*A button in focused state.*

Focus states are inherited by action, selection, and input components:
- Buttons
- Cards
- Checkbox
- Chips
- Date and time pickers
- List items
- Selection controls
- Text fields

*Focus state of 4 components.*

Focus states aren’t inherited by most communication, containment, or navigation components:
- App bars
- Badges
- Banner
- Card
- Dialogs
- Navigation bar, drawer, and rail
- Sheets

*Mobile screen showing  entire app bar in focus state, which is an error.*

Keyboard focus indicator
Many people use the Tab key or other shortcut to navigate the interactive elements of a web page, like links, buttons, and chips.
When an element is tabbed to, it appears in its focused state with a ring-like keyboard focus indicator. This indicator helps web users know where they are on the page.
While focused, an element can be acted on with the keyboard.

*A single filled button in focused state, displaying the keyboard focus indicator.*

Behavior
Focus states are initiated by the user by pressing the Tab key on the keyboard (or equivalent).
Focus states can be represented in combination with hover, activated, or selected states.

*A single filter chip simultaneously in selected state and focus state.*

There can only be one focus state at a time in a layout.

*Cursor moving from one card in focus state to another card moves the focus state to the second card.*

Activated

Activated states indicate which item from a set of options is currently being viewed. They are initiated either by default or user choice, using input methods such as a tap, cursor, keyboard, or voice input. 
Activated states are higher emphasis and signified by an overlay, color change, or other visual treatments applied to elements or segments within a component.

*The activated tab has a different color and shape than the enabled tabs.*

An activated state differs from a selected state because it communicates a highlighted destination.
Activation states are inherited by items within some navigation components: 
✓ Navigation bar, drawer, and rail
✓ Tabs

*The tab and navigation drawer are both in activated states.*

Activation states aren’t inherited by action, communication, containment, selection, or input components: 
✗ App bars
✗ Badges
✗ Buttons
✗ Checkbox
✗ Chips
✗ Dialogs
✗ Sheets
✗ Sliders
✗ Switch
✗ Text fields

*Activated state applied to a button in error.*

Behavior
Activated states can be represented in combination with hover and focus states.

*A cursor hovers over unselected button in a hovered state.  On a click it switches to active state.*

Activated states can be represented in combination with hover and focus states.

*Screen showing inbox in focus and activated states.*

Within a single set of options, only one activated state may be present at a time.

*Email settings navigation list “inbox”is activated. The selection components has one activated state as well.*

Pressed

A pressed state communicates a user-initiated tap or click via cursor, keyboard, or voice input. This state applies to all interactive components.
Pressed states trigger a change in composition and should be high-emphasis.
A ripple overlay signifies a pressed state. It can be applied to an entire component or elements within a component, or as a circular shape over part of the component.

*Button using a ripple overlay to signify it’s in a pressed state.*

Some components, such as buttons or cards, can inherit elevation to signify a pressed state.

*Button using elevation to signify it’s in a pressed state.*

Pressed states are inherited by action, selection, and some containment components: 
- Buttons
- Cards
- Checkbox
- Chips
- List items
- Text fields

*Four components shown in pressed state.*

Pressed states aren’t inherited by communication, navigation, or some containment components: 
- App bars
- Badges
- Bottom navigation
- Dialogs
- Menus
- Sheets
- Tabs

*Mobile screen showing  entire app bar in pressed state is an error.*

Behavior
Pressed states are initiated by user keyboard or voice input on an interactive element.

*Enabled state activated to pressed state.*

Pressed states can be combined with hovered, focused, activated, or selected states.

*Hovered state activated to a pressed state.*

There may only be a single pressed state at a time in a layout.

*Pressed state on one card at a time.*

Dragged

A dragged state occurs when a user presses and moves an element or component. Dragged states should be low emphasis, to avoid distracting users from their task.
Dragged states use a lower emphasis overlay. It can be applied to the entire component or to elements within a component.
Some components, such as list items, chips, or cards, can inherit elevation to signify a dragged state.

*List item shown in dragged state.*

Dragged states are inherited by some containment and selection components: 
- Cards
- Chips
- List items
- Sliders

*A chip and a card both shown in dragged state.*

Dragged states aren’t inherited by action, communication, navigation, or some containment components: 
- App bars
- Badges
- Buttons
- Dialogs
- Menus
- Navigation bar, drawer, and rail

*Mobile screen with app bar in dragged state is an error.*

Behavior
Dragged states are initiated when users touch and hold elements, using an input method such as a tap or click.

*Going through the states of a draggable list item:  enabled, hovered, dragged.*

There may only be a single dragged state at a time within a layout.

*Cursor dragging cards one at a time.*
