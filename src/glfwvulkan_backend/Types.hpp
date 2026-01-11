#define GLFW_INCLUDE_VULKAN
#pragma once
#include <glfw/glfw3.h>
#include <vulkan/vulkan.hpp>
#include <memory>
#include <string>
#include <functional>

typedef uint32_t AurenID;

struct AurenVec2D {
    int width = 0;
    int height = 0;
};

struct WindowDeleter {
    void operator()(GLFWwindow* window) const {
        glfwDestroyWindow(window);
    }
};

class AurenWindow {
    private:
    static void _framebufferResizeCallback(GLFWwindow* window, int width, int height);
    std::function<void(GLFWwindow* window, int width, int height)> _rendererResizeCallback;

    AurenVec2D _size {};
    bool _framebufferResized = false;
    std::string _windowName = "";
    std::unique_ptr<GLFWwindow, WindowDeleter> _window = nullptr;
    VkSurfaceKHR _surface = nullptr;
    VkInstance _instance = nullptr;
    public:

    AurenWindow(std::string title, AurenVec2D size, VkInstance instance);
    ~AurenWindow();

    AurenWindow(const AurenWindow&) = delete;
    AurenWindow& operator=(const AurenWindow&) = delete;
    AurenWindow(AurenWindow&& other) noexcept;

    inline bool shouldClose() { return glfwWindowShouldClose(_window.get()); }
    inline void pollEvents() { glfwPollEvents(); }

    void setResizeCallback(std::function<void(GLFWwindow*, int, int)> callback);

    GLFWwindow* refGLFWwindow() const { 
        return _window.get(); 
    }

    VkSurfaceKHR getSurface() { return _surface; }
    VkExtent2D getExtent() { return { (uint32_t)_size.width, (uint32_t)_size.height }; }

    bool wasResized() { return _framebufferResized; }
    void resetResizedFlag() { _framebufferResized = false; }


};

AurenWindow::AurenWindow(AurenWindow&& other) noexcept
    :   _window(std::move(other._window)),
        _instance(other._instance),
        _surface(other._surface),
        _size(other._size),
        _windowName(std::move(other._windowName)),
        _rendererResizeCallback(std::move(other._rendererResizeCallback))
    {
    
    other._surface = VK_NULL_HANDLE;

    if (_window) {
        glfwSetWindowUserPointer(_window.get(), this);
    }
}

AurenWindow::AurenWindow(std::string title, AurenVec2D size, VkInstance instance) 
    : _windowName(title), _size(size), _instance(instance) {
    
    glfwWindowHint(GLFW_CLIENT_API, GLFW_NO_API);
    
    GLFWwindow* rawPtr = glfwCreateWindow(size.width, size.height, title.c_str(), nullptr, nullptr);
    if (!rawPtr) throw std::runtime_error("Failed to create GLFW window");
    
    _window.reset(rawPtr);

    glfwSetWindowUserPointer(_window.get(), this);
    glfwSetFramebufferSizeCallback(_window.get(), _framebufferResizeCallback);

    if (glfwCreateWindowSurface(_instance, _window.get(), nullptr, &_surface) != VK_SUCCESS) {
        throw std::runtime_error("Failed to create window surface!");
    }
}

AurenWindow::~AurenWindow() {
    if (_surface != VK_NULL_HANDLE) {
        vkDestroySurfaceKHR(_instance, _surface, nullptr);
    }
}

void AurenWindow::setResizeCallback(std::function<void(GLFWwindow*, int, int)> callback) {
    _rendererResizeCallback = callback;
}

void AurenWindow::_framebufferResizeCallback(GLFWwindow* window, int width, int height) {
    auto app = reinterpret_cast<AurenWindow*>(glfwGetWindowUserPointer(window));
    if (app) {
        app->_size = { width, height };
        app->_framebufferResized = true;
        
        if (app->_rendererResizeCallback) {
            app->_rendererResizeCallback(window, width, height);
        }
    }
}