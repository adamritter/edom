TODO:
    + Implement render_if
    - Recursive foreach
    - Implement inmemorydom
    - Convert examples from other frameworks, show comparisions.
    - Improve class names (Iterator to visitor, others easier after different files/namespaces)
    - Tutorial
    - fetch
    - HN clone
    - Store Event (check if event was already subscribed to?)
    - Svelte like templates
    + <input type=number bind:value={a} min=0 max=10>
    + <input type=range bind:value={a} min=0 max=10>
    + <input type=checkbox bind:checked={yes}>
    + <input type=radio bind:group={scoops} name="scoops" value={1}>
    + <textarea bind:value={value}></textarea>
    - <select bind:value={selected} on:change="{() => answer = ''}">
        - <option value={question}> text
            Note that the <option> values are objects rather than strings. Svelte doesn't mind.

    - A select can have a multiple attribute, in which case it will populate an array rather than selecting a single value.

    - <div
	    contenteditable="true"
	    bind:innerHTML={html}
        ></div>
    - You can even bind to properties inside an each block.
    - document get_dnode()
    - timer events
    - animation:  <progress value={$progress}></progress> https://svelte.dev/tutorial/tweened
    - fade transition



