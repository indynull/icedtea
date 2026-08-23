# Color roles

There are 26 standard color roles organized into six groups: primary, secondary, tertiary, error, surface, and outline

## Tab 1

What are color roles?
Color roles are like the "numbers" in a paint-by-number canvas. They're the connective tissue between elements of the UI and what color goes where.
- Color roles are mapped to Material Components
You'll use these color roles whether you're using the static baseline scheme or dynamic color. If your product contains custom components, they'll need to be properly mapped to this set of color roles.
- Color roles ensure accessibility
The color system is built on accessible color pairings. These color pairs provide an accessible minimum 3:1 contrast.
- Color roles are tokenized
Roles are implemented in design and code through tokens. A design token represents a small, reusable design decision that’s part of a design system's visual style.

*Example color swatches for all 45 color roles including Primary, Secondary, Tertiary, Error, Surfaces, Inverse roles, Scrim and Shadow roles.*

General concepts
Here are helpful-to-know words you'll see in the names of color roles:
- Surface – A role used for backgrounds and large, low-emphasis areas of the screen.
- Primary, Secondary, Tertiary – Accent color roles used to emphasize or de-emphasize foreground elements.
- Container – Roles used as a fill color for foreground elements like buttons. They should not be used for text or icons.
- On – Roles starting with this term indicate a color for text or icons on top of its paired parent color. For example, on primary is used for text and icons against the primary fill color.
- Variant – Roles ending with this term offer a lower emphasis alternative to its non-variant pair. For example, outline variant is a less emphasized version of the outline color.

Pairing and layering colors
To ensure accessible visual contrast in your app, apply colors only in the intended pairs or layering orders described in the following sections.
Combining colors improperly may break contrast necessary for visual accessibility, particularly when colors are adjusted through dynamic color features such as user-controlled contrast.

*Pair of buttons shown at three different contrast levels. At every level, the text on the button is legible because the color roles are properly applied to the buttons.*

*Pair of buttons shown at 3 different contrast levels. The text on the button becomes illegible because the color roles are improperly applied to the buttons.*

Accent color roles

Accent color roles include primary, secondary, and tertiary. Assign them to elements in the UI based on importance and needed emphasis. Use caution when changing color roles for visual effect.

- Primary roles are for important actions and elements needing the most emphasis, like a FAB to start a new message.

- Secondary roles are for elements that don’t need immediate attention and don’t need emphasis, like the selected state of a navigation icon or a dismissive button.

- Tertiary roles are for smaller elements that need special emphasis but don't require immediate attention, such as a badge or notification.

*A mail app showing a mix of primary, secondary, and tertiary accent colors used across the screen.*

Primary

Use primary roles for the most prominent components across the UI, such as the FAB, high-emphasis buttons, and active states.

- Primary: High-emphasis fills, texts, and icons against surface

- On primary: Text and icons against primary

- Primary container: Standout fill color against surface, for key components like FAB

- On primary container: Text and icons against primary container

*4 color swatches: primary, on primary, primary container and on primary container.*

*Primary color used for filled button, and "on primary" color used for the text within that button.*

*"Primary container" color used for FAB and "on primary container" color used for the text and icon within.*

Secondary

Use secondary roles for less prominent components in the UI such as filter chips.

There are four secondary roles:
- Secondary: Less prominent fills, text, and icons against surface

- On secondary: Text and icons against secondary

- Secondary container: Less prominent fill color against surface, for recessive components like tonal buttons

- On secondary container: Text and icons against secondary container

*4 color swatches: secondary, on secondary, secondary container, and on secondary container.*

*An icon button using "secondary container" color, and an icon using "on secondary color."*

Tertiary

Use tertiary roles for contrasting accents that balance primary and secondary colors or bring heightened attention to an element such as an input field.

There are four tertiary roles:
- Tertiary: Complementary fills, text, and icons against surface

- On tertiary: Text and icons against tertiary

- Tertiary container: Complementary container color against surface, for components like input fields

- On tertiary container: Text and icons against tertiary container

*4 color swatches: tertiary, on tertiary, tertiary container, and on tertiary container.*

The tertiary color roles can be applied at the designer's discretion. They're intended to support broader color expression.

*A selected element using Tertiary Container as a background color and On Tertiary Container for text.*

Error

Use error roles to communicate error states, such as an incorrect password entered into a text field.

There are four error roles:
- Error: Attention-grabbing color against surface for fills, icons, and text, indicating urgency

- On error: Text and icons against error

- Error container: Attention-grabbing fill color against surface

- On error container: Text and icons against error container
Error is an example of a static color (it doesn't change even in dynamic color schemes). Error color roles are made static by default with any dynamic color scheme. They still adapt to light and dark theme.

*4 color swatches: Error, On error, Error container and On error container roles.*

Surface

Use surface roles for more neutral backgrounds, and container colors for components like cards, sheets, and dialogs.

There are three surface roles:
- Surface: Default color for backgrounds

- On surface: Text and icons against any surface or surface container color

- On surface variant: Lower-emphasis color for text and icons against any surface or surface container color

*3 color swatches: Surface, On surface and On surface variant.*

There are also five surface container roles named based on their level of emphasis:
- Surface container lowest: Lowest-emphasis container color

- Surface container low: Low-emphasis container color

- Surface container: Default container color

- Surface container high: High-emphasis container color

- Surface container highest: Highest-emphasis container color

Surface container is the default role, but the others are especially helpful for creating hierarchy and nested containers in layouts for expanded screens.

*4 color swatches in light and dark theme: Surface container lowest, Surface container low, Surface container, Surface container high and Surface container highest.*

The most common combination of surface roles uses surface for a background area and surface container for a navigation area.
Text and icons typically use on surface and on surface variant on all types of surfaces.

*Email app using Surface for the main background color and Surface Container for the navigation bar background*

All color mappings – but especially surface colors – should remain the same for layout regions across breakpoints. For example, the body area will use the surface color and the navigation area will use the surface container color on both mobile and tablet.

*Mobile and tablet screens both using Surface for main background and surface container for navigation background.*

Depending on necessary hierarchy, feature area, and design logic, you can use add-on surface colors in larger window class sizes as long as colors are consistently applied.

*Mobile, foldable, and tablet screens showing how the body and navigation regions have the same color roles across window sizes.*

By default, neutral-colored components such as navigation bars, menus, or dialogs are mapped to specific surface container roles, but these roles can be remapped by makers to suit user needs.

*Surface container low applied to an elevated button and card, surface container applied to the top and bottom bar, surface container high applied to the FAB and basic dialog, surface container highest applied to an input label and off switch.*

Inverse colors
Inverse roles are applied selectively to components to achieve colors that are the reverse of those in the surrounding UI, creating a contrasting effect.
- Inverse surface: Background fills for elements which contrast against surface

- Inverse on surface: Text and icons against inverse surface

- Inverse primary: Actionable elements, such as text buttons, against inverse surface

*3 color swatches: Inverse surface, inverse on surface and inverse primary roles.*

*Snackbar component using inverse surface for its background, inverse on surface for its text and inverse primary of its text button, shown in light theme*

Outline

There are two outline colors to be used against a surface:
- Outline: Important boundaries, such as a text field outline

- Outline variant: Decorative elements, such as dividers, and when other elements provide 4.5:1 contrast

*2 color swatches: Outline and outline variant*

*Diagram comparing used of outline and outline variant.*

*Outline color incorrectly used for dividers*

*Outline color incorrectly used for cards.*

*Outline variant color incorrectly used for input and filter chips.*

*Outline variant incorrectly color used for chips.*

Add-on color roles

Most products won't need to use these add-on color roles. However, some products require the greater flexibility and control that add-on roles provide. If you aren't sure whether your product should use the add-on roles, it probably shouldn't and you can ignore them.

Fixed accent colors
Primary fixed, secondary fixed, and tertiary fixed are fill colors used against surface. These colors maintain the same tone in light and dark themes, as opposed to regular container colors, which change in tone between these themes. The fixed color role may be used instead of the equivalent container role in situations where such fixed behavior is desired.
The primary fixed dim, secondary fixed dim, and tertiary fixed dim roles provide a stronger, more emphasized tone relative to the equivalent fixed color. They may be used where a deeper color but the same fixed behavior is desired.

*6 color swatches: Primary, secondary and tertiary fixed swatches, along with their darker Dim counterparts, shown in both light and dark theme.*

*FAB in light and dark theme, using the primary fixed role for its container fill color.*

*FAB in light and dark theme, using the primary container role for its container fill color.*

*2 email app screens using primary fixed and primary fixed dim.*

*Permissions screen with Surface used for the background and Primary Fixed incorrectly used for the button fill.*

*Permissions screen with Surface used for the background and Primary correctly used for the button fill.*

On fixed accent colors
On fixed colors are used for text and icons which sit on top of the corresponding Fixed color. For example, on primary fixed is used for text and icons against the primary fixed color. The same usage applies for the equivalent secondary and tertiary colors.

On fixed variant colors are used for text and icons needing lower emphasis against the corresponding fixed color. For example, on primary fixed variant is used for low emphasis text and icons against the primary fixed color. The same usage applies for the equivalent secondary and tertiary colors.

*6 color swatches for on fixed and on fixed variant*

*Banner in a news app using on primary fixed variant and on primary fixed*

Bright and dim surface roles
There are two add-on surface roles:
- Surface dim: Dimmest surface color in light and dark themes
- Surface bright: Brightest surface color in light and dark themes

*3 color swatches: Surface dim, surface, and surface bright*

*3 color swatches: Surface dim, surface, and surface bright*

While the default surface color automatically inverts between light and dark themes (it’s a light color in light theme and it flips to a dark color in dark theme), the surface bright and surface dim colors invert in a slightly different way. More precisely, they keep their relative brightness across both light and dark theme.
For example, in an interface using the default surface role, the mapped area is the brightest in light theme and the dimmest in dark theme. In an interface using the surface bright role, the mapped area is the brightest in both light and dark theme.

*UI with surface role applied to the body area and surface container applied to the navigation area, shown in light theme.*

*UI with surface role applied to the body area and surface container applied to the navigation area, shown in dark theme.*

*UI with surface bright role applied to the body area and surface container applied to the navigation area, shown in light theme.*

*UI with surface bright role applied to the body area and surface container applied to the navigation area, shown in dark theme.*

*Large screen chat UI. The surface dim role is applied to the left navigation rail and the surface bright role is applied to the chat window.*
