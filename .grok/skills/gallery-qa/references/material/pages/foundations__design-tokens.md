# Design tokens

Design tokens are the building blocks of all UI elements. The same tokens are used in designs, tools, and code.

## Overview

- Tokens point to style values like colors, fonts, and measurements

- Use design tokens instead of hardcoded values

- Each token is named for how or where it’s used (for example, md.comp.fab.primary.container.color sets the container color for a FAB)

- Even if a token’s end value is changed, its name and use remain the same

- Material Design has three classes of tokens: reference, system, and component

*FAB container color token in a design mock, code, and product*

Resources

| | Type| Link| Status

| Design| Design Kit (Figma)| Available
| Material Theme Builder Figma plugin| Available
| Implementation| Material baseline theme and tokens (DSP)| Available

What’s a design token?

Design tokens are small, reusable design decisions that make up a design system's visual style. Tokens replace static values with self-explanatory names.
A design token consists of 2 things:
- A code-like name, such as md.ref.palette.secondary90

- An associated value, such as #E8DEF8

The token's value can be one of several things: A color, typeface, measurement, or even another token.

*Token md.ref.palette.secondary90 with arrow pointing to lilac color swatch #E8DEF8.*

Design tokens meaningfully connect style choices that would otherwise lack a clear relationship.
For example, if a designer's mock-ups and an engineer's implementation both reference the same token for the “secondary container color,” then they can be confident that the same color is being used in both places. This applies even if the hex value assigned to that token gets updated.

*Diagram of FAB indicating system tokens: Secondary for surface color and on secondary for icon color*

Why are tokens important?

Tokens make it possible for a design system to have a single source of truth – a repository where style choices are recorded and changes can be tracked.

Because tokens are reusable and purpose-driven, they can define system-wide updates to themes and contexts. For example, you can use tokens to systematically apply a high-contrast color palette for improved visibility, or change the typographic scale to ensure that text is legible on a TV screen.

By using tokens for design and implementation, style updates propagate consistently through an entire product or suite of products. They also help designers and engineers "speak the same language,” reducing confusion during handoff from design to implementation.

*Diagram of how changing the value of a token cascades from design system into a product area and 3 products*

Deciding if tokens are right for you

Tokens will be most helpful if:
- You plan to update the design of your product or are building a product from scratch
- Your design system is applied across a suite of products or platforms
- You want to make it easy to maintain or update styles in the future
- You want to get the most out of the Material Design system, including features like dynamic color

Tokens will be less helpful if:
- You have an existing app using hard-coded values that is unlikely to change in the next year or two
- Your product does not have a design system

Tokens & Material Design

In the past, Material styles were communicated through guidelines, design files, tools, and platform-specific component libraries.

With design tokens, you can now download, customize, and apply Material styles and integrate them across your design and development process.

Tokens allow decisions to be documented in a platform-agnostic and shareable format.

*Diagram of design token connected to design tool and platform code*

On this site, you’ll see tokens listed in interactive modules.

These modules let you quickly look up the default baseline value stored by tokens for color, font, font size, font weight, etc. They also show the relationship between a role, its system token, reference token, and stored pre-set value.

*A token module for the color system with 4 color swatches for the primary color set.*

You’ll also see tokens in the specs tabs of component articles.

Tokens are first grouped by state (enabled, disabled, hover, etc) and then by element, which is the part of the component that a token or value applies to, such as the container or label text.

Columns include:
- Name – The component style aspect that the token applies to, such as color or font
- Token ID – The token defining the component style aspect
- Description – Optional descriptive info
- Context/value – The value stored in the token for a given context

*A token module for an elevated button organized by state (enabled, disabled, etc), then element (container, label text).*

How to use token modules
Let's say you need to verify the color role for a filled button's label text.
Navigate to Common buttons > Specs, find the token module for filled buttons, and search for the "label text" tokens under elements.
Copy the color token and paste it in code, or compare it to the color role in Figma.

*How to find the label text color token.*

Parts of a token name

The parts of a token name are separated by periods and proceed from the most general information ("md") to the most specific ("on-secondary").
- All token names in a design system start with the system name (such as “md” for Material Design)
- An abbreviation for the token class: “ref” for reference tokens, “sys” for system tokens, and “comp” for component tokens
- The token ends with descriptive words communicating the token’s role

*Diagram of the 3 parts of a design token name: system prefix, token type, and role description*

Classes of tokens

There are three classes of tokens in Material:
- Reference tokens
All available tokens with associated values.
- System tokens
Decisions and roles that give the design system its character, from color and typography, to elevation and shape.
- Component tokens
The design properties assigned to elements in a component, such as the color of a button icon.
With three classes of tokens, teams can update design decisions globally or apply a change to a single component.

*Color value stored in reference token stored in system token, stored in component token, appearing as the container color of a FAB*

Reference tokens
These tokens make up all of the style options available in a design system.
They usually point to a static value – such as a color hex code or font size – but can also point to other reference tokens. Reference tokens don't change based on context.
By providing a list of options, reference tokens give your team a starting point of approved colors, typography, measurements, etc.
All reference tokens start with the prefix ref.

*Token md.ref.palette.secondary90 pointing to #E8DEF8. Toke md.ref.typeface.plain-medium pointing to Roboto Medium*

System tokens
These are the decisions that systematize the design language for a specific theme or context.
System tokens define the purpose a reference token serves in the UI.
This is where theming occurs. The system token can point to different reference tokens depending on the context, such as a light or dark theme.
Whenever possible, system tokens should point to reference tokens rather than static values. 
All system tokens start with the prefix sys.

*System token md.sys.color.secondary-container pointing to token md.ref.palette.secondary90 pointing to #E8DEF8. System token md.sys.typescale.label-medium.font pointing to token md.ref.typeface.plain-medium pointing to Roboto Medium*

Component tokens (in development)
These represent the elements required to compose a component, such as containers, label text, icons, states, and their values such as size, shape, color, or elevation.
Whenever possible, component tokens should point to a system or reference token, and not contain hardcoded values such as hex codes.
Not every stylistic choice of a component will be able to be expressed as a token, but whenever a design choice applies to multiple components of similar intent, a token should be used.
All component tokens start with the prefix comp.

*Extended FAB component tokens for container color and label text pointing to system tokens, pointing to reference tokens, pointing to resolved values*

Contexts

Tokens can point to different values depending on a set of conditions. These conditions are called contexts and their resulting values are called contextual values.

Examples of different contexts include: device form factors, dark theme, dense layouts, and right-to-left writing systems.

You can think of a context as a kind of tag. If a token value is tagged with dark theme then it will override the default token value in a dark theme context.

*Diagram of system token for background color pointing to different values depending on the theme context*

How to use tokens

Download Material baseline tokens
Material’s baseline theme includes design tokens and default values. Download the theme as a Design System Package (DSP) to customize, collaborate on, and use in your own designs and product code. Learn about the DSP JSON format

Use tokens in Figma
To begin, install the Material Theme Builder Figma plugin from the community page.

Generate tokens
- Open Figma and navigate to: Plugins > Material Theme Builder > Open Plugin
- Select Get started, this will create material-theme with baseline values by default. Color and text styles will begin populating the right hand design panel. When your tokens are fully generated, your artboard will contain tonal palettes for light and dark color schemes, as well as a default type scale.
- Your tokens are now represented as Figma styles that can be used throughout your designs

Import tokens via DSP
- Open Figma and navigate to: Plugins > Material Theme Builder > Open Plugin
- Open the theme dropdown and select Import DSP
- Select the token DSP file from your list of files. Color and text styles will begin populating the right hand design panel. When your tokens are fully imported, your artboard will contain tonal palettes for light and dark color schemes, as well as a default type scale.
- Your tokens are now represented as Figma styles that can be used throughout your designs

Update token values
Using the Material Theme Builder Figma plugin (updates colors only)
- Open Figma and navigate to: Plugins > Material Theme Builder > Open Plugin
- With Custom selected, choose colors. Updated color and text styles will begin populating the right hand design panel.
- Your updated tokens are now represented as Figma styles that can be used throughout your designs

Using Figma styles
- In Figma, navigate to the file in which the tokenized style is defined. Shortcut: right click the style in the right hand sidebar and select Go to style definition.
- In the right hand sidebar, hover over the style you want to update and select the adjust icon when it appears. Or, right click the style in the style picker and select Edit style.
- Make your changes to the token name, description, properties, etc. using the Edit style panel. Close the panel when finished.

Use tokens in product mock-ups
Instead of manually setting the color or typography for elements in a layout, apply the Figma styles representing your design tokens. This helps ensure that developers will correctly understand and apply your design choices.

Use tokens with the Material Design Kit
- Duplicate the Material Design Kit in Figma
- Navigate to Plugins > Material Theme Builder > Open plugin
- With components selected, select swap. This will swap the baseline Material token style values with your own generated token style values.

Export tokens
- Open Figma and navigate to: Plugins > Material Theme Builder > Open Plugin
- Navigate to the Export tab. Select the format you want to export for (Android, Jetpack Compose, DSP)
- Name your .zip file and select Save
Your tokens are ready to share!

## How to use tokens

Download Material baseline tokens

Material Design’s baseline theme includes design tokens and default values. Download the theme as a Design System Package (DSP) to customize, collaborate on, and use in your own designs and product code.  Learn about the DSP JSON format

Use tokens in Figma

To begin, install the Material Theme Builder Figma plugin from the community page.

Generate tokens
- Open Figma and navigate to: Plugins > Material Theme Builder > Open Plugin
- Select Get started, this will create material-theme with baseline values by default. Color and text styles will begin populating the right hand design panel. When your tokens are fully generated, your artboard will contain tonal palettes for light and dark color schemes, as well as a default type scale.
- Your tokens are now represented as Figma styles that can be used throughout your designs

Update token values

Using the Material Theme Builder Figma plugin (updates colors only)
- Open Figma and navigate to: Plugins > Material Theme Builder > Open Plugin
- Choose the colors. Updated color and text styles will begin populating the right hand design panel.
- Your updated tokens are now represented as Figma styles that can be used throughout your designs

Using Figma styles
- In Figma, navigate to the file in which the tokenized style is defined. Shortcut: right click the style in the right hand sidebar and select Go to style definition.
- In the right hand sidebar, hover over the style you want to update and select the adjust icon when it appears. Or, right click the style in the style picker and select Edit style.
- Make your changes to the token name, description, properties, etc. using the Edit style panel. Close the panel when finished.

Use tokens in product mock-ups
- Instead of manually setting the color or typography for elements in a layout, apply the Figma styles representing your design tokens. This helps ensure that developers will correctly understand and apply your design choices.

Use tokens with the Material Design Kit
- Duplicate the Material Design Kit in Figma
- Navigate to Plugins > Material Theme Builder > Open plugin
- With components selected, select swap. This will swap the baseline Material token style values with your own generated token style values.

Export tokens
- Open Figma and navigate to: Plugins > Material Theme Builder > Open Plugin
- Navigate to the Export tab. Select the format you want to export for (Android, Jetpack Compose)
- Name your .zip file and select Save
Your tokens are ready to share!

Use tokens in VSCode

To begin, install the Adobe XD extension for Visual Studio Code. This enables you to directly integrate a DSP file into your product.

Import tokens
- Get the DSP file from your designer or download the Material baseline theme
- Unzip the DSP file
- Open VSCode. Drag the opened DSP folder into the environment window, copying it into your workspace

Update token values
- Navigate to your DSP file, dsp.json.
- Open the Adobe XD extension and select Tokens from the menu. This shows you a list of all tokens and token values.
- Select Start Editing
- Find the token you want to edit and select the pencil icon to the right of its card
- Make any changes to the token type, ID, value, or description
- Select the checkbox to the right of the token’s card
- Select Finish Editing, which recompiles the tokens
- Hot refresh your code. You’ll be able to see the updated token reflected in your preview.

Learn more about the Adobe XD extension for VSCode
- Video: The Adobe XD Visual Studio Code Plugin (Crash Course) by DesignCourse
- Playlist: Adobe XD x VSCODE Pro Tips by Matt Rae
