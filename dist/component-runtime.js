/**
 * RavensOne Component System
 *
 * A React-like component system for building reusable UI elements
 * Features:
 * - Props and state management
 * - Lifecycle methods (mount, update, unmount)
 * - Virtual DOM diffing
 * - Event handling
 * - Reactive updates
 */

class Component {
    constructor(props = {}) {
        this.props = props;
        this.state = {};
        this.el = null;
        this.children = [];
        this.isMounted = false;
        this.updateScheduled = false;
        this._eventListeners = [];
    }

    /**
     * Set component state and trigger re-render
     */
    setState(updates) {
        const prevState = { ...this.state };

        // Merge updates
        if (typeof updates === 'function') {
            this.state = { ...this.state, ...updates(this.state) };
        } else {
            this.state = { ...this.state, ...updates };
        }

        // Schedule update
        if (this.isMounted && !this.updateScheduled) {
            this.updateScheduled = true;
            requestAnimationFrame(() => {
                this.updateScheduled = false;
                this.componentDidUpdate(this.props, prevState);
                this.update();
            });
        }
    }

    /**
     * Render method - must be overridden by subclasses
     */
    render() {
        throw new Error('Component must implement render()');
    }

    /**
     * Lifecycle: Component mounted
     */
    componentDidMount() {
        // Override in subclass
    }

    /**
     * Lifecycle: Component updated
     */
    componentDidUpdate(prevProps, prevState) {
        // Override in subclass
    }

    /**
     * Lifecycle: Component will unmount
     */
    componentWillUnmount() {
        // Override in subclass
    }

    /**
     * Mount component to DOM
     */
    mount(container) {
        if (typeof container === 'string') {
            container = document.querySelector(container);
        }

        if (!container) {
            throw new Error('Mount container not found');
        }

        const rendered = this.render();
        this.el = this._createElement(rendered);
        container.appendChild(this.el);
        this.isMounted = true;
        this.componentDidMount();

        return this;
    }

    /**
     * Update component
     */
    update() {
        if (!this.el || !this.isMounted) {
            return;
        }

        const rendered = this.render();
        const newEl = this._createElement(rendered);

        this.el.replaceWith(newEl);
        this.el = newEl;
    }

    /**
     * Unmount component
     */
    unmount() {
        if (!this.el) {
            return;
        }

        this.componentWillUnmount();

        // Remove event listeners
        for (const { el, event, handler } of this._eventListeners) {
            el.removeEventListener(event, handler);
        }
        this._eventListeners = [];

        this.el.remove();
        this.el = null;
        this.isMounted = false;
    }

    /**
     * Create DOM element from virtual node
     */
    _createElement(vnode) {
        // Handle text nodes
        if (typeof vnode === 'string' || typeof vnode === 'number') {
            return document.createTextNode(String(vnode));
        }

        // Handle null/undefined
        if (!vnode) {
            return document.createTextNode('');
        }

        // Handle arrays
        if (Array.isArray(vnode)) {
            const fragment = document.createDocumentFragment();
            for (const child of vnode) {
                fragment.appendChild(this._createElement(child));
            }
            return fragment;
        }

        // Handle component instances
        if (vnode instanceof Component) {
            return vnode._createElement(vnode.render());
        }

        // Handle element objects
        const { tag, props = {}, children = [] } = vnode;
        const el = document.createElement(tag);

        // Set attributes and properties
        for (const [key, value] of Object.entries(props)) {
            if (key === 'className') {
                el.className = value;
            } else if (key === 'style' && typeof value === 'object') {
                Object.assign(el.style, value);
            } else if (key.startsWith('on') && typeof value === 'function') {
                const event = key.substring(2).toLowerCase();
                el.addEventListener(event, value);
                this._eventListeners.push({ el, event, handler: value });
            } else if (key === 'ref' && typeof value === 'function') {
                value(el);
            } else {
                el.setAttribute(key, value);
            }
        }

        // Add children
        for (const child of children) {
            el.appendChild(this._createElement(child));
        }

        return el;
    }
}

/**
 * Create virtual DOM element
 */
function h(tag, props = {}, ...children) {
    return {
        tag,
        props,
        children: children.flat(),
    };
}

/**
 * Component Library - Reusable UI Components
 */

class Button extends Component {
    render() {
        const {
            label = 'Button',
            onClick = () => {},
            variant = 'primary',
            disabled = false,
            size = 'medium',
        } = this.props;

        const baseStyles = {
            padding: size === 'small' ? '8px 16px' : size === 'large' ? '16px 32px' : '12px 24px',
            fontSize: size === 'small' ? '12px' : size === 'large' ? '16px' : '14px',
            fontWeight: '600',
            border: 'none',
            borderRadius: '8px',
            cursor: disabled ? 'not-allowed' : 'pointer',
            transition: 'all 0.2s',
            opacity: disabled ? '0.5' : '1',
        };

        const variantStyles = {
            primary: {
                background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
                color: 'white',
            },
            secondary: {
                background: '#6c757d',
                color: 'white',
            },
            danger: {
                background: '#dc3545',
                color: 'white',
            },
            success: {
                background: '#28a745',
                color: 'white',
            },
            outline: {
                background: 'transparent',
                color: '#667eea',
                border: '2px solid #667eea',
            },
        };

        return h('button', {
            className: `ravens-button ravens-button-${variant} ravens-button-${size}`,
            style: { ...baseStyles, ...variantStyles[variant] },
            onClick: disabled ? undefined : onClick,
            disabled,
        }, label);
    }
}

class Card extends Component {
    render() {
        const {
            title,
            children = [],
            footer,
            padding = '20px',
        } = this.props;

        const cardStyle = {
            background: 'white',
            borderRadius: '15px',
            boxShadow: '0 10px 30px rgba(0,0,0,0.1)',
            overflow: 'hidden',
        };

        const bodyStyle = {
            padding,
        };

        return h('div', { className: 'ravens-card', style: cardStyle }, [
            title && h('div', {
                style: {
                    padding: '20px',
                    borderBottom: '2px solid #f0f0f0',
                    fontWeight: '600',
                    fontSize: '18px',
                    color: '#333',
                }
            }, title),
            h('div', { className: 'ravens-card-body', style: bodyStyle }, children),
            footer && h('div', {
                style: {
                    padding: '15px 20px',
                    borderTop: '2px solid #f0f0f0',
                    background: '#f8f9fa',
                }
            }, footer),
        ]);
    }
}

class Modal extends Component {
    constructor(props) {
        super(props);
        this.state = {
            isOpen: props.isOpen || false,
        };
    }

    open() {
        this.setState({ isOpen: true });
    }

    close() {
        this.setState({ isOpen: false });
        if (this.props.onClose) {
            this.props.onClose();
        }
    }

    render() {
        const { title, children = [], footer, width = '500px' } = this.props;
        const { isOpen } = this.state;

        if (!isOpen) {
            return h('div', { style: { display: 'none' } });
        }

        const overlayStyle = {
            position: 'fixed',
            top: '0',
            left: '0',
            width: '100%',
            height: '100%',
            background: 'rgba(0,0,0,0.5)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: '1000',
            animation: 'fadeIn 0.3s ease',
        };

        const modalStyle = {
            background: 'white',
            borderRadius: '20px',
            boxShadow: '0 20px 60px rgba(0,0,0,0.3)',
            width,
            maxWidth: '90vw',
            maxHeight: '90vh',
            overflow: 'hidden',
            animation: 'slideIn 0.3s ease',
        };

        return h('div', {
            className: 'ravens-modal-overlay',
            style: overlayStyle,
            onClick: () => this.close(),
        }, [
            h('div', {
                className: 'ravens-modal',
                style: modalStyle,
                onClick: (e) => e.stopPropagation(),
            }, [
                h('div', {
                    style: {
                        padding: '20px',
                        borderBottom: '2px solid #f0f0f0',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'space-between',
                    }
                }, [
                    h('h2', { style: { margin: '0', fontSize: '20px' } }, title),
                    h('button', {
                        style: {
                            background: 'none',
                            border: 'none',
                            fontSize: '24px',
                            cursor: 'pointer',
                            color: '#999',
                        },
                        onClick: () => this.close(),
                    }, '×'),
                ]),
                h('div', { style: { padding: '20px', maxHeight: '60vh', overflow: 'auto' } }, children),
                footer && h('div', {
                    style: {
                        padding: '15px 20px',
                        borderTop: '2px solid #f0f0f0',
                        background: '#f8f9fa',
                    }
                }, footer),
            ]),
        ]);
    }
}

class Input extends Component {
    render() {
        const {
            type = 'text',
            placeholder = '',
            value = '',
            onChange = () => {},
            disabled = false,
            label,
        } = this.props;

        const inputStyle = {
            width: '100%',
            padding: '12px',
            border: '2px solid #e0e0e0',
            borderRadius: '8px',
            fontSize: '14px',
            transition: 'border-color 0.2s',
        };

        const elements = [];

        if (label) {
            elements.push(h('label', {
                style: {
                    display: 'block',
                    marginBottom: '8px',
                    color: '#666',
                    fontWeight: '600',
                    fontSize: '14px',
                }
            }, label));
        }

        elements.push(h('input', {
            type,
            placeholder,
            value,
            disabled,
            style: inputStyle,
            onInput: (e) => onChange(e.target.value),
        }));

        return h('div', { className: 'ravens-input-group' }, elements);
    }
}

class Badge extends Component {
    render() {
        const {
            children,
            variant = 'primary',
            size = 'medium',
        } = this.props;

        const variantColors = {
            primary: '#667eea',
            success: '#28a745',
            danger: '#dc3545',
            warning: '#ffc107',
            info: '#17a2b8',
        };

        const style = {
            display: 'inline-block',
            padding: size === 'small' ? '4px 8px' : '6px 12px',
            borderRadius: '20px',
            fontSize: size === 'small' ? '10px' : '12px',
            fontWeight: '600',
            background: variantColors[variant],
            color: 'white',
        };

        return h('span', { className: `ravens-badge ravens-badge-${variant}`, style }, children);
    }
}

class List extends Component {
    render() {
        const {
            items = [],
            renderItem = (item) => String(item),
            keyExtractor = (item, index) => index,
            emptyMessage = 'No items',
        } = this.props;

        if (items.length === 0) {
            return h('div', {
                style: {
                    textAlign: 'center',
                    padding: '40px',
                    color: '#999',
                }
            }, emptyMessage);
        }

        return h('ul', {
            className: 'ravens-list',
            style: {
                listStyle: 'none',
                padding: '0',
                margin: '0',
            }
        }, items.map((item, index) =>
            h('li', {
                key: keyExtractor(item, index),
                style: {
                    padding: '12px',
                    marginBottom: '8px',
                    background: '#f8f9fa',
                    borderRadius: '8px',
                    transition: 'all 0.2s',
                }
            }, renderItem(item, index))
        ));
    }
}

// Export
if (typeof window !== 'undefined') {
    window.RavensComponents = {
        Component,
        h,
        Button,
        Card,
        Modal,
        Input,
        Badge,
        List,
    };
    console.log('[Components] Runtime initialized');
}

if (typeof module !== 'undefined' && module.exports) {
    module.exports = {
        Component,
        h,
        Button,
        Card,
        Modal,
        Input,
        Badge,
        List,
    };
}
