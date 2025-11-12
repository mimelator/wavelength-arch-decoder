# AI Assistant Chat UI

Interactive web-based chat interface for the Architecture Decoder AI Assistant.

## Features

- 🎨 **Beautiful Dark Theme** - Modern, easy-on-the-eyes interface
- 💬 **Real-time Chat** - Interactive conversation with the AI assistant
- 📊 **Rich Results Display** - Shows sources, graph statistics, and related entities
- 🔍 **Repository Selection** - Easy dropdown to select which repository to query
- 📱 **Responsive Design** - Works on desktop and mobile devices

## Usage

1. Start the AI Assistant server:
   ```bash
   cd ai-assistant
   ./start.sh
   ```

2. Open your browser to:
   ```
   http://localhost:8000
   ```

3. Select a repository from the dropdown

4. Start asking questions!

## Example Queries

- "What functions are available?"
- "What services are used?"
- "What dependencies does this use?"
- "What functions use Firebase?"
- "What would break if I rename getAdminStorage?"

## Display Features

The UI displays:
- **AI Answers** - Formatted responses from OpenAI
- **Sources** - Code elements, services, dependencies found
- **Knowledge Graph Statistics** - Node counts, edge counts, node types
- **Related Entities** - Services and dependencies related to the query

## Files

- `index.html` - Main HTML structure
- `style.css` - Dark theme styling
- `app.js` - Interactive functionality and API integration

