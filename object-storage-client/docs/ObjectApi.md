# \ObjectApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**append_object**](ObjectApi.md#append_object) | **PUT** /objects/{object_id}/append | 
[**create_object**](ObjectApi.md#create_object) | **POST** /objects | 
[**delete_object**](ObjectApi.md#delete_object) | **DELETE** /objects/{object_id} | 
[**get_object_by_id**](ObjectApi.md#get_object_by_id) | **GET** /objects/{object_id} | 
[**get_object_by_path**](ObjectApi.md#get_object_by_path) | **GET** /objects/by-path | 
[**get_objects**](ObjectApi.md#get_objects) | **GET** /objects | 
[**move_object**](ObjectApi.md#move_object) | **PUT** /objects/{object_id}/move | 
[**read_object_by_id**](ObjectApi.md#read_object_by_id) | **GET** /objects/{object_id}/read | 
[**read_object_by_path**](ObjectApi.md#read_object_by_path) | **GET** /objects/by-path/read | 



## append_object

> models::UploadResponse append_object(object_id, part)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**object_id** | **i64** |  | [required] |
**part** | **std::path::PathBuf** |  | [required] |

### Return type

[**models::UploadResponse**](UploadResponse.md)

### Authorization

[Authorization](../README.md#Authorization)

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_object

> models::ObjectInstance create_object(create_object_request)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_object_request** | [**CreateObjectRequest**](CreateObjectRequest.md) |  | [required] |

### Return type

[**models::ObjectInstance**](ObjectInstance.md)

### Authorization

[Authorization](../README.md#Authorization)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_object

> delete_object(object_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**object_id** | **i64** |  | [required] |

### Return type

 (empty response body)

### Authorization

[Authorization](../README.md#Authorization)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_object_by_id

> models::ObjectInstance get_object_by_id(object_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**object_id** | **i64** |  | [required] |

### Return type

[**models::ObjectInstance**](ObjectInstance.md)

### Authorization

[Authorization](../README.md#Authorization)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_object_by_path

> models::ObjectInstance get_object_by_path(path)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**path** | **String** |  | [required] |

### Return type

[**models::ObjectInstance**](ObjectInstance.md)

### Authorization

[Authorization](../README.md#Authorization)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_objects

> models::Pagination get_objects(offset, limit, path)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**offset** | Option<**u32**> |  |  |
**limit** | Option<**u32**> |  |  |
**path** | Option<**String**> |  |  |

### Return type

[**models::Pagination**](Pagination.md)

### Authorization

[Authorization](../README.md#Authorization)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## move_object

> models::ObjectInstance move_object(object_id, move_object_request)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**object_id** | **i64** |  | [required] |
**move_object_request** | [**MoveObjectRequest**](MoveObjectRequest.md) |  | [required] |

### Return type

[**models::ObjectInstance**](ObjectInstance.md)

### Authorization

[Authorization](../README.md#Authorization)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## read_object_by_id

> read_object_by_id(object_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**object_id** | **i64** |  | [required] |

### Return type

 (empty response body)

### Authorization

[Authorization](../README.md#Authorization)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: */*, application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## read_object_by_path

> read_object_by_path(path)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**path** | **String** |  | [required] |

### Return type

 (empty response body)

### Authorization

[Authorization](../README.md#Authorization)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: */*, application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

